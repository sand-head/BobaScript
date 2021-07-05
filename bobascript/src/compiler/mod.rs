use std::rc::Rc;

use thiserror::Error;

use self::{
  parser::Parser,
  rules::{ParseRule, Precedence},
  scanner::{Token, TokenType},
};
use crate::{
  chunk::{Chunk, JumpDirection, OpCode},
  debug::disassemble_chunk,
  parse_both, parse_infix, parse_none, parse_prefix,
  value::{Function, Value},
};

mod parser;
mod rules;
mod scanner;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Error, Clone)]
pub enum CompileError {
  #[error("Unexpected character '{1}' on line {0}.")]
  UnexpectedCharacter(usize, char),
  #[error("Unterminated string on line {0}.")]
  UnterminatedString(usize),
  #[error("Expected {0}.")]
  Expected(&'static str),
  #[error("Invalid assignment target.")]
  InvalidAssignmentTarget,
  #[error("A variable with the name \"{0}\" already exists in this scope.")]
  VariableAlreadyExists(String),
  #[error("A variable with the name \"{0}\" does not exist in scope.")]
  VariableDoesNotExist(String),
  #[error("Functions and function calls can only have a maximum of 255 arguments. Why do you need that many?")]
  TooManyArguments,
  #[error("Cannot return from top-level code.")]
  TopLevelReturn,
}

struct Local {
  name: Token,
  // todo: change this so we don't use -1 for uninitialized locals
  depth: i32,
}

pub enum FunctionType {
  /// The root (or top) level script.
  TopLevel,
  /// A function within the script.
  Function,
}
pub struct CompileContext {
  function: Function,
  fn_type: FunctionType,
  locals: Vec<Local>,
  scope_depth: i32,
}
impl CompileContext {
  pub fn new(fn_type: FunctionType) -> Self {
    Self {
      function: Function::default(),
      fn_type,
      locals: vec![Local {
        name: Token {
          token_type: TokenType::Identifier,
          lexeme: "".to_string(),
          line: 0,
        },
        depth: 0,
      }],
      scope_depth: 0,
    }
  }
}

pub struct Compiler {
  contexts: Vec<CompileContext>,
  parser: Parser,
}
impl Compiler {
  pub fn new() -> Self {
    Self {
      contexts: vec![CompileContext::new(FunctionType::TopLevel)],
      parser: Parser::new(String::from("")),
    }
  }

  pub fn compile(&mut self, source: String) -> CompileResult<Rc<Function>> {
    self.parser = Parser::new(source);

    self.parser.advance();
    loop {
      match self.parser.current_type() {
        Some(TokenType::EOF) => break self.parser.advance(),
        _ => self.statement(),
      }
    }
    let function = self.end_compiler();

    match &self.parser.error {
      None => Ok(function),
      Some(err) => Err(err.clone()),
    }
  }

  fn push_context(&mut self, fn_type: FunctionType) {
    self.contexts.push(CompileContext {
      function: Function::default(),
      fn_type,
      locals: vec![Local {
        name: Token {
          token_type: TokenType::Identifier,
          lexeme: "".to_string(),
          line: 0,
        },
        depth: 0,
      }],
      scope_depth: 0,
    });
  }

  fn pop_context(&mut self) -> Option<CompileContext> {
    self.contexts.pop()
  }

  fn current_context(&self) -> &CompileContext {
    &self.contexts.last().unwrap()
  }

  fn current_context_mut(&mut self) -> &mut CompileContext {
    self.contexts.last_mut().unwrap()
  }

  fn current_chunk(&mut self) -> &mut Chunk {
    &mut self.contexts.last_mut().unwrap().function.chunk
  }

  fn scope_depth(&self) -> i32 {
    self.contexts.last().unwrap().scope_depth
  }

  fn scope_depth_mut(&mut self) -> &mut i32 {
    &mut self.contexts.last_mut().unwrap().scope_depth
  }

  fn locals(&self) -> &Vec<Local> {
    &self.contexts.last().unwrap().locals
  }

  fn locals_mut(&mut self) -> &mut Vec<Local> {
    &mut self.contexts.last_mut().unwrap().locals
  }

  fn parse_precedence(&mut self, precedence: Precedence) {
    self.parser.advance();
    let rule_prefix = get_rule(self.parser.previous_type().unwrap()).prefix;

    let can_assign = precedence <= Precedence::Assignment;
    match rule_prefix {
      Some(prefix) => prefix(self, can_assign),
      None => self.parser.set_error(CompileError::Expected("expression")),
    }

    if can_assign && self.parser.current_type().unwrap() == TokenType::Assign {
      self.parser.advance();
      self.parser.set_error(CompileError::InvalidAssignmentTarget);
    }

    while precedence <= get_rule(self.parser.current_type().unwrap()).precedence {
      self.parser.advance();
      let rule_infix = get_rule(self.parser.previous_type().unwrap()).infix;
      if let Some(infix) = rule_infix {
        infix(self, can_assign);
      }
    }
  }

  fn statement(&mut self) {
    match self.parser.current_type() {
      Some(TokenType::Const) => {
        todo!("add const statement")
      }
      Some(TokenType::Let) => {
        // skip past "let" token:
        self.parser.advance();
        self.declaration();
      }
      Some(TokenType::Return) => {
        self.parser.advance();
        self.return_stmt();
      }
      Some(_) => {
        self.expression();
        let previous_type = self.parser.previous_type();
        let current_type = self.parser.current_type();

        match (previous_type, current_type) {
          (_, Some(TokenType::RightBrace)) if self.scope_depth() > 0 => {
            // do nothing
            // this allows us to use the last expression in a block in assigning new variables
            // ex: let test = { let test2 = 15; test2 / 3 };
          }
          (Some(TokenType::RightBrace), _) if current_type != Some(TokenType::Semicolon) => {
            // do not advance, but DO emit a pop
            // we get here if a block has just closed
            // we don't want to have to put semicolons after just any plain ol' block now, do we?
            // however, we also don't want stray values emitted from the block in the stack
            // ex: { let test = "howdy!"; log test; }
            self.emit_opcode(OpCode::Pop);
          }
          (_, Some(TokenType::Semicolon)) => {
            // standard statement ending, just do the usual
            self.parser.advance();
            self.emit_opcode(OpCode::Pop);
          }
          _ => self
            .parser
            .set_error(CompileError::Expected("';' after expression")),
        }
      }
      _ => unreachable!(),
    }

    if self.parser.is_panicking() {
      self.parser.synchronize();
    }
  }

  fn declaration(&mut self) {
    let global = self.parse_variable(CompileError::Expected("variable name"));

    if let Some(TokenType::Assign) = self.parser.current_type() {
      self.parser.advance();
      self.expression();
    } else {
      self.emit_opcode(OpCode::Unit);
    }
    self.parser.consume(
      TokenType::Semicolon,
      CompileError::Expected("';' after let statement"),
    );

    self.define_variable(global);
  }

  fn return_stmt(&mut self) {
    if let FunctionType::TopLevel = self.current_context().fn_type {
      self.parser.set_error(CompileError::TopLevelReturn);
    }

    if let Some(TokenType::Semicolon) = self.parser.current_type() {
      self.parser.advance();
      self.emit_opcode(OpCode::Return);
    } else {
      self.expression();
      self.parser.consume(
        TokenType::Semicolon,
        CompileError::Expected("';' after return value"),
      );
      self.emit_opcode(OpCode::Return);
    }
  }

  fn expression(&mut self) {
    self.parse_precedence(Precedence::Assignment);
  }

  fn log_expr(&mut self) {
    self.expression();
    self.emit_opcode(OpCode::Log);
  }

  fn and(&mut self) {
    let end_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));

    self.emit_opcode(OpCode::Pop);
    self.parse_precedence(Precedence::And);

    self.patch_jump(end_jump);
  }

  fn or(&mut self) {
    let else_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
    let end_jump = self.emit_opcode_idx(OpCode::Jump(JumpDirection::Forwards, 0));

    self.patch_jump(else_jump);
    self.emit_opcode(OpCode::Pop);

    self.parse_precedence(Precedence::Or);
    self.patch_jump(end_jump);
  }

  fn grouping(&mut self) {
    self.expression();
    self.parser.consume(
      TokenType::RightParen,
      CompileError::Expected("')' after expression"),
    );
  }

  fn block(&mut self) {
    self.begin_scope();

    loop {
      match self.parser.current_type().unwrap() {
        TokenType::RightBrace | TokenType::EOF => break,
        _ => self.statement(),
      }
    }

    // hack: if the last statement in the block ended in a semicolon,
    // OR if there weren't any statements and it was an empty block,
    // emit a unit so that the expression always has a value
    if let Some(TokenType::Semicolon | TokenType::LeftBrace) = self.parser.previous_type() {
      self.emit_opcode(OpCode::Unit);
    }

    self.parser.consume(
      TokenType::RightBrace,
      CompileError::Expected("'}' after block"),
    );

    self.end_scope();
  }

  fn fn_expr(&mut self) {
    // let compiler = Compiler::child(Rc::new(RefCell::new(self)), FunctionType::Function);
    // this *might* work without having to bring unsafe rust into it... fingers crossed!
    self.push_context(FunctionType::Function);
    self.begin_scope();

    self.parser.consume(
      TokenType::LeftParen,
      CompileError::Expected("'(' after \"fn\" keyword"),
    );
    if self.parser.current_type() != Some(TokenType::RightParen) {
      // there's something in these dang parentheses
      loop {
        if self.current_context().function.arity == u8::MAX {
          // todo: throw too many parameters error
        } else {
          self.current_context_mut().function.arity += 1;
        }

        // parse the parameter
        let idx = self.parse_variable(CompileError::Expected("parameter name"));
        self.define_variable(idx);

        if self.parser.current_type() != Some(TokenType::Comma) {
          // break if no more parameters
          break;
        } else {
          self.parser.advance();
        }
      }
    }
    self.parser.consume(
      TokenType::RightParen,
      CompileError::Expected("')' after parameters"),
    );
    self.parser.consume(
      TokenType::LeftBrace,
      CompileError::Expected("block after parameters"),
    );
    self.block();
    self.emit_opcode(OpCode::Return);

    let context = self.pop_context().unwrap();
    let idx = self.make_constant(Value::Function(Rc::new(context.function)));
    self.emit_opcode(OpCode::Constant(idx));
  }

  fn if_expr(&mut self) {
    self.expression();
    let then_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
    self.emit_opcode(OpCode::Pop);

    self.parser.consume(
      TokenType::LeftBrace,
      CompileError::Expected("block after \"if\" expression"),
    );
    self.block();

    let else_jump = self.emit_opcode_idx(OpCode::Jump(JumpDirection::Forwards, 0));
    self.patch_jump(then_jump);
    self.emit_opcode(OpCode::Pop);

    if self.parser.current_type() == Some(TokenType::Else) {
      self.parser.advance();
      // todo: check for "if" keyword, for else ifs
      self.parser.consume(
        TokenType::LeftBrace,
        CompileError::Expected("block after \"else\" clause"),
      );
      self.block();
    }

    self.patch_jump(else_jump);
  }

  fn while_expr(&mut self) {
    let loop_start = self.current_chunk().code.len();
    self.expression();

    let exit_jump = self.emit_opcode_idx(OpCode::JumpIfFalse(0));
    self.emit_opcode(OpCode::Pop);

    self.parser.consume(
      TokenType::LeftBrace,
      CompileError::Expected("block after \"while\" loop"),
    );
    self.block();
    // since this lang uses rust-like loops and blocks and things,
    // we have to acknowledge that, as much as I may enjoy the trailing expressions in blocks,
    // they cannot exist here, in while loops
    // therefore, we must reject every value from the block
    self.emit_opcode(OpCode::Pop);
    self.emit_loop(loop_start);

    self.patch_jump(exit_jump);
    self.emit_opcode(OpCode::Pop);

    // ...however, since this *is* still an expression, it must return *something*
    self.emit_opcode(OpCode::Unit);
  }

  fn unary(&mut self) {
    let unary_operator = self.parser.previous_type().unwrap();

    // compile the operand
    self.parse_precedence(Precedence::Unary);

    // emit the operator
    match unary_operator {
      TokenType::Minus => self.emit_opcode(OpCode::Negate),
      TokenType::Not => self.emit_opcode(OpCode::Not),
      _ => unreachable!(),
    }
  }

  fn binary(&mut self) {
    let binary_operator = self.parser.previous_type().unwrap();
    let rule: usize = get_rule(binary_operator).precedence.into();
    self.parse_precedence((rule + 1).into());

    match binary_operator {
      TokenType::Asterisk => self.emit_opcode(OpCode::Multiply),
      TokenType::Carrot => self.emit_opcode(OpCode::Exponent),
      TokenType::Minus => self.emit_opcode(OpCode::Subtract),
      TokenType::Plus => self.emit_opcode(OpCode::Add),
      TokenType::Slash => self.emit_opcode(OpCode::Divide),
      TokenType::NotEqual => {
        self.emit_opcode(OpCode::Equal);
        self.emit_opcode(OpCode::Not);
      }
      TokenType::Equal => self.emit_opcode(OpCode::Equal),
      TokenType::GreaterThan => self.emit_opcode(OpCode::GreaterThan),
      TokenType::GreaterEqual => {
        self.emit_opcode(OpCode::LessThan);
        self.emit_opcode(OpCode::Not);
      }
      TokenType::LessThan => self.emit_opcode(OpCode::LessThan),
      TokenType::LessEqual => {
        self.emit_opcode(OpCode::GreaterThan);
        self.emit_opcode(OpCode::Not);
      }
      _ => unreachable!(),
    }
  }

  fn call(&mut self) {
    let arg_count = self.argument_list();
    self.emit_opcode(OpCode::Call(arg_count));
  }

  fn argument_list(&mut self) -> u8 {
    let mut arg_count = 0;
    if self.parser.current_type() != Some(TokenType::RightParen) {
      loop {
        self.expression();
        if arg_count == u8::MAX {
          self.parser.set_error(CompileError::TooManyArguments);
        } else {
          arg_count += 1;
        }

        if self.parser.current_type() == Some(TokenType::Comma) {
          self.parser.advance();
        } else {
          break;
        }
      }
    }
    self.parser.consume(
      TokenType::RightParen,
      CompileError::Expected("')' after arguments"),
    );
    arg_count
  }

  fn literal(&mut self) {
    match self.parser.previous_type() {
      Some(TokenType::Unit) => self.emit_opcode(OpCode::Unit),
      Some(TokenType::False) => self.emit_opcode(OpCode::False),
      Some(TokenType::True) => self.emit_opcode(OpCode::True),
      _ => unreachable!(),
    }
  }

  fn variable(&mut self, can_assign: bool) {
    self.named_variable(self.parser.previous().unwrap().lexeme.clone(), can_assign)
  }

  fn string(&mut self) {
    let string = &self.parser.previous().unwrap().lexeme;
    // strip the leading and trailing quotation mark off the lexeme:
    let string = string[1..(string.len() - 1)].to_string();
    let string_idx = self.make_constant(Value::String(string));
    self.emit_opcode(OpCode::Constant(string_idx))
  }

  fn number(&mut self) {
    let num = &self.parser.previous().unwrap().lexeme;
    let num = num.parse::<f64>().unwrap();
    let num_idx = self.make_constant(Value::Number(num));
    self.emit_opcode(OpCode::Constant(num_idx));
  }

  fn make_constant(&mut self, value: Value) -> usize {
    self.current_chunk().add_constant(value)
  }

  fn emit_opcode(&mut self, opcode: OpCode) {
    let line_no = self.parser.previous().unwrap().line;
    self.current_chunk().write(opcode, line_no);
  }

  /// Emits the given `OpCode` and returns its index in the chunk.
  fn emit_opcode_idx(&mut self, opcode: OpCode) -> usize {
    let line_no = self.parser.previous().unwrap().line;
    self.current_chunk().write(opcode, line_no)
  }

  /// This just emits a `Jump` instruction, but backwards
  fn emit_loop(&mut self, start: usize) {
    let offset = self.current_chunk().code.len() - start + 1;
    self.emit_opcode(OpCode::Jump(JumpDirection::Backwards, offset));
  }

  fn patch_jump(&mut self, offset: usize) {
    let new_jump = self.current_chunk().code.len() - 1 - offset;
    let (opcode, line) = &self.current_chunk().code[offset];
    self.current_chunk().code[offset] = (
      match opcode {
        OpCode::Jump(direction, _) => OpCode::Jump(*direction, new_jump),
        OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(new_jump),
        _ => unreachable!(),
      },
      *line,
    );
  }

  fn begin_scope(&mut self) {
    *self.scope_depth_mut() += 1;
  }

  fn end_scope(&mut self) {
    *self.scope_depth_mut() -= 1;

    let mut count: usize = 0;
    for i in (0..self.locals().len()).rev() {
      if self.locals()[i].depth > self.scope_depth() {
        self.locals_mut().remove(i);
        count += 1;
      } else {
        break;
      }
    }
    if count == 1 {
      self.emit_opcode(OpCode::Pop);
    } else if count > 1 {
      self.emit_opcode(OpCode::PopN(count));
    }
  }

  fn parse_variable(&mut self, err: CompileError) -> usize {
    self.parser.consume(TokenType::Identifier, err);
    self.declare_variable();

    if self.scope_depth() > 0 {
      0
    } else {
      self.identifier_constant(self.parser.previous().unwrap().lexeme.clone())
    }
  }

  fn mark_initialized(&mut self) {
    let idx = self.locals().len() - 1;
    self.locals_mut()[idx].depth = self.scope_depth();
  }

  /// Initializes a variable in the scope for use
  fn define_variable(&mut self, global: usize) {
    if self.scope_depth() > 0 {
      self.mark_initialized();
    } else {
      self.emit_opcode(OpCode::DefineGlobal(global));
    }
  }

  /// Adds a variable to the scope
  fn declare_variable(&mut self) {
    if self.scope_depth() > 0 {
      let name = self.parser.previous().unwrap().clone();
      // todo: this sucks
      let mut name_exists = false;
      for local in self.locals().iter().rev() {
        if local.depth != -1 && local.depth < self.scope_depth() {
          break;
        }
        if &name.lexeme == &local.name.lexeme {
          name_exists = true;
          break;
        }
      }

      if name_exists {
        self
          .parser
          .set_error(CompileError::VariableAlreadyExists(name.lexeme.clone()));
      } else {
        self.locals_mut().push(Local { name, depth: -1 });
      }
    }
  }

  fn identifier_constant(&mut self, lexeme: String) -> usize {
    self.make_constant(Value::String(lexeme))
  }

  fn resolve_local(&mut self, name: &String) -> Option<usize> {
    for i in (0..self.locals().len()).rev() {
      if name == &self.locals()[i].name.lexeme {
        if self.locals()[i].depth == -1 {
          self.parser.set_error(CompileError::VariableDoesNotExist(
            self.locals()[i].name.lexeme.clone(),
          ));
        }
        // println!("yeah we got a {} at index {}", name, i);
        return Some(i);
      }
    }
    None
  }

  fn named_variable(&mut self, lexeme: String, can_assign: bool) {
    let (get_op, set_op) = if let Some(idx) = self.resolve_local(&lexeme) {
      (OpCode::GetLocal(idx), OpCode::SetLocal(idx))
    } else {
      let idx = self.identifier_constant(lexeme);
      (OpCode::GetGlobal(idx), OpCode::SetGlobal(idx))
    };

    match self.parser.current_type() {
      Some(TokenType::Assign) if can_assign => {
        // skip assign token, parse an expression, and make it this variable's value
        self.parser.advance();
        self.expression();
        self.emit_opcode(set_op);
      }
      Some(TokenType::AddAssign) if can_assign => {
        // skip assign token, parse an expression, and make it this variable's value
        self.parser.advance();
        self.emit_opcode(get_op);
        self.expression();
        self.emit_opcode(OpCode::Add);
        self.emit_opcode(set_op);
      }
      Some(TokenType::SubtractAssign) if can_assign => {
        // skip assign token, parse an expression, and make it this variable's value
        self.parser.advance();
        self.emit_opcode(get_op);
        self.expression();
        self.emit_opcode(OpCode::Subtract);
        self.emit_opcode(set_op);
      }
      Some(TokenType::MultiplyAssign) if can_assign => {
        // skip assign token, parse an expression, and make it this variable's value
        self.parser.advance();
        self.emit_opcode(get_op);
        self.expression();
        self.emit_opcode(OpCode::Multiply);
        self.emit_opcode(set_op);
      }
      Some(TokenType::DivideAssign) if can_assign => {
        // skip assign token, parse an expression, and make it this variable's value
        self.parser.advance();
        self.emit_opcode(get_op);
        self.expression();
        self.emit_opcode(OpCode::Divide);
        self.emit_opcode(set_op);
      }
      _ => self.emit_opcode(get_op),
    }
  }

  fn end_compiler(&mut self) -> Rc<Function> {
    self.emit_opcode(OpCode::Unit);
    self.emit_opcode(OpCode::Return);
    let context = self.contexts.pop().unwrap();

    if crate::DEBUG && self.parser.error.is_none() {
      disassemble_chunk(
        &context.function.chunk,
        if context.function.name.len() > 0 {
          &context.function.name
        } else {
          "[script]"
        },
      );
    }

    Rc::new(context.function)
  }
}

/// Gets the appropriate `ParseRule` for the given `TokenType`.
fn get_rule(token_type: TokenType) -> ParseRule {
  match token_type {
    TokenType::LeftParen => parse_both!(|c, _| c.grouping(), |c, _| c.call(), Call),
    TokenType::LeftBrace => parse_prefix!(|c, _| c.block(), None),

    TokenType::Asterisk => parse_infix!(|c, _| c.binary(), Factor),
    TokenType::Carrot => parse_infix!(|c, _| c.binary(), Exponent),
    TokenType::Minus => parse_both!(|c, _| c.unary(), |c, _| c.binary(), Term),
    TokenType::Plus => parse_infix!(|c, _| c.binary(), Term),
    TokenType::Slash => parse_infix!(|c, _| c.binary(), Factor),

    TokenType::Not => parse_prefix!(|c, _| c.unary(), None),
    TokenType::NotEqual => parse_infix!(|c, _| c.binary(), Equality),
    TokenType::Equal => parse_infix!(|c, _| c.binary(), Equality),
    TokenType::GreaterThan => parse_infix!(|c, _| c.binary(), Comparison),
    TokenType::GreaterEqual => parse_infix!(|c, _| c.binary(), Comparison),
    TokenType::LessThan => parse_infix!(|c, _| c.binary(), Comparison),
    TokenType::LessEqual => parse_infix!(|c, _| c.binary(), Comparison),

    TokenType::Unit => parse_prefix!(|c, _| c.literal(), None),
    TokenType::Identifier => parse_prefix!(|c, can_assign| c.variable(can_assign), None),
    TokenType::String => parse_prefix!(|c, _| c.string(), None),
    TokenType::Number => parse_prefix!(|c, _| c.number(), None),

    TokenType::And => parse_infix!(|c, _| c.and(), And),
    TokenType::False => parse_prefix!(|c, _| c.literal(), None),
    TokenType::Fn => parse_prefix!(|c, _| c.fn_expr(), None),
    TokenType::If => parse_prefix!(|c, _| c.if_expr(), None),
    TokenType::Log => parse_prefix!(|c, _| c.log_expr(), None),
    TokenType::Or => parse_infix!(|c, _| c.or(), Or),
    TokenType::True => parse_prefix!(|c, _| c.literal(), None),
    TokenType::While => parse_prefix!(|c, _| c.while_expr(), None),

    _ => parse_none!(),
  }
}
