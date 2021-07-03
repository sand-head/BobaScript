use thiserror::Error;

use self::{
  parser::Parser,
  rules::{ParseRule, Precedence},
  scanner::{Token, TokenType},
};
use crate::{
  chunk::{Chunk, OpCode},
  debug::disassemble_chunk,
  parse_both, parse_infix, parse_none, parse_prefix,
  value::Value,
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
}

struct Local {
  name: Token,
  // todo: change this so we don't use -1 for uninitialized locals
  depth: i32,
}

pub struct Compiler<'a> {
  parser: Parser,
  chunk: &'a mut Chunk,
  locals: Vec<Local>,
  scope_depth: i32,
}
impl<'a> Compiler<'a> {
  pub fn new(chunk: &'a mut Chunk) -> Self {
    Self {
      parser: Parser::new(String::from("")),
      chunk,
      locals: Vec::new(),
      scope_depth: 0,
    }
  }

  pub fn compile(&mut self, source: String) -> CompileResult<()> {
    self.parser = Parser::new(source);

    self.parser.advance();
    match self.parser.current_type() {
      Some(TokenType::EOF) => self.parser.advance(),
      _ => self.statement(),
    }
    self.end_compiler();

    match &self.parser.error {
      None => Ok(()),
      Some(err) => Err(err.clone()),
    }
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
      Some(TokenType::Log) => {
        self.parser.advance();
        self.expression();
        self.parser.consume(
          TokenType::Semicolon,
          CompileError::Expected("';' after value"),
        );
        self.emit_opcode(OpCode::Log);
      }
      Some(TokenType::Const) => {
        todo!("add const statement")
      }
      Some(TokenType::Let) => {
        // skip past "let" token:
        self.parser.advance();
        self.let_statement();
      }
      Some(TokenType::LeftBrace) => {
        // todo: make blocks expressions instead of statements
        self.parser.advance();
        self.begin_scope();
        self.block();
        self.end_scope();
      }
      Some(TokenType::If) => {
        // todo: also make ifs expressions instead of statements
        self.parser.advance();
        self.if_statement();
      }
      Some(_) => {
        self.expression();
        self.parser.consume(
          TokenType::Semicolon,
          CompileError::Expected("';' after expression"),
        );
        self.emit_opcode(OpCode::Pop);
      }
      _ => unreachable!(),
    }

    if self.parser.is_panicking() {
      self.parser.synchronize();
    }
  }

  fn let_statement(&mut self) {
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

  fn expression(&mut self) {
    self.parse_precedence(Precedence::Assignment);
  }

  fn grouping(&mut self) {
    self.expression();
    self.parser.consume(
      TokenType::RightParen,
      CompileError::Expected("')' after expression"),
    );
  }

  fn block(&mut self) {
    loop {
      match self.parser.current_type().unwrap() {
        TokenType::RightBrace | TokenType::EOF => break,
        _ => self.statement(),
      }
    }

    self.parser.consume(
      TokenType::RightBrace,
      CompileError::Expected("'}' after block"),
    );
  }

  fn if_statement(&mut self) {
    self.expression();
    let then_jump = self.emit_jump(OpCode::JumpIfFalse(0));
    self.emit_opcode(OpCode::Pop);

    self.parser.consume(
      TokenType::LeftBrace,
      CompileError::Expected("block after if statement"),
    );
    self.begin_scope();
    self.block();
    self.end_scope();

    let else_jump = self.emit_jump(OpCode::Jump(0));
    self.patch_jump(then_jump);
    self.emit_opcode(OpCode::Pop);

    if self.parser.current_type() == Some(TokenType::Else) {
      self.parser.advance();
      // todo: check for "if" keyword, for else ifs
      self.parser.consume(
        TokenType::LeftBrace,
        CompileError::Expected("block after else statement"),
      );
      self.begin_scope();
      self.block();
      self.end_scope();
    }

    self.patch_jump(else_jump);
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
    self.chunk.add_constant(value)
  }

  fn emit_opcode(&mut self, opcode: OpCode) {
    self
      .chunk
      .write(opcode, self.parser.previous().unwrap().line);
  }

  fn emit_jump(&mut self, opcode: OpCode) -> usize {
    self
      .chunk
      .write(opcode, self.parser.previous().unwrap().line)
  }

  fn patch_jump(&mut self, offset: usize) {
    let new_jump = self.chunk.code.len() - 1 - offset;
    let (opcode, line) = &self.chunk.code[offset];
    self.chunk.code[offset] = (
      match opcode {
        OpCode::Jump(_) => OpCode::Jump(new_jump),
        OpCode::JumpIfFalse(_) => OpCode::JumpIfFalse(new_jump),
        _ => unreachable!(),
      },
      *line,
    );
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) {
    self.scope_depth -= 1;

    let mut count: usize = 0;
    for i in (0..self.locals.len()).rev() {
      if self.locals[i].depth > self.scope_depth {
        self.locals.remove(i);
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

    if self.scope_depth > 0 {
      0
    } else {
      self.identifier_constant(self.parser.previous().unwrap().lexeme.clone())
    }
  }

  fn mark_initialized(&mut self) {
    let idx = self.locals.len() - 1;
    self.locals[idx].depth = self.scope_depth;
  }

  /// Initializes a variable in the scope for use
  fn define_variable(&mut self, global: usize) {
    if self.scope_depth > 0 {
      self.mark_initialized();
    } else {
      self.emit_opcode(OpCode::DefineGlobal(global));
    }
  }

  /// Adds a variable to the scope
  fn declare_variable(&mut self) {
    if self.scope_depth > 0 {
      let name = self.parser.previous().unwrap().clone();
      for local in self.locals.iter().rev() {
        if local.depth != -1 && local.depth < self.scope_depth {
          break;
        }
        if &name.lexeme == &local.name.lexeme {
          self
            .parser
            .set_error(CompileError::VariableAlreadyExists(name.lexeme.clone()));
        }
      }

      self.locals.push(Local { name, depth: -1 });
    }
  }

  fn identifier_constant(&mut self, lexeme: String) -> usize {
    self.make_constant(Value::String(lexeme))
  }

  fn resolve_local(&mut self, name: &String) -> Option<usize> {
    for i in (0..self.locals.len()).rev() {
      if name == &self.locals[i].name.lexeme {
        if self.locals[i].depth == -1 {
          self.parser.set_error(CompileError::VariableDoesNotExist(
            self.locals[i].name.lexeme.clone(),
          ));
        }
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

    if can_assign && self.parser.current_type() == Some(TokenType::Assign) {
      // skip assign token, parse an expression, and make it this variable's value
      self.parser.advance();
      self.expression();
      self.emit_opcode(set_op);
    } else {
      self.emit_opcode(get_op);
    }
  }

  fn end_compiler(&mut self) {
    self.emit_opcode(OpCode::Return);
    if crate::DEBUG && self.parser.error.is_none() {
      disassemble_chunk(self.chunk, "code");
    }
  }
}

/// Gets the appropriate `ParseRule` for the given `TokenType`.
fn get_rule(token_type: TokenType) -> ParseRule {
  match token_type {
    TokenType::LeftParen => parse_prefix!(|c, _| c.grouping(), None),

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

    TokenType::False => parse_prefix!(|c, _| c.literal(), None),
    TokenType::True => parse_prefix!(|c, _| c.literal(), None),

    _ => parse_none!(),
  }
}
