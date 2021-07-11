use std::rc::Rc;

use bobascript_parser::ast::{Ast, Expr, Stmt};

use super::{CompileContext, CompileError, CompileResult, FunctionType, Local};
use crate::{
  chunk::{Chunk, JumpDirection, OpCode, Upvalue},
  debug::disassemble_chunk,
  value::{Function, Value},
};

pub struct Compiler {
  contexts: Vec<CompileContext>,
}
impl Compiler {
  pub fn new() -> Self {
    Self {
      contexts: vec![CompileContext::new(FunctionType::TopLevel)],
    }
  }

  pub(super) fn compile(&mut self, ast: &Ast) -> CompileResult<Rc<Function>> {
    for stmt in ast {
      self.statement();
    }

    self.emit_opcode(OpCode::Tuple(0));
    let function = self.end_compiler();
    self.parser.error.clone().map_or(Ok(function), |e| Err(e))
  }

  /// Compiles a single expression and returns its containing function.
  pub(super) fn compile_expr(&mut self, expr: &Box<Expr>) -> CompileResult<Rc<Function>> {
    match **expr {
      Expr::Log(_) => todo!(),
      Expr::Block(_, _) => todo!(),
      Expr::If(_, _, _) => todo!(),
      Expr::While(_, _) => todo!(),
      Expr::Binary(_, _, _) => todo!(),
      Expr::Unary(_, _) => todo!(),
      Expr::Call(_, _) => todo!(),
      Expr::Constant(_) => todo!(),
    };

    let function = self.end_compiler();
    self.parser.error.clone().map_or(Ok(function), |e| Err(e))
  }

  fn push_context(&mut self, fn_type: FunctionType) {
    self.contexts.push(CompileContext::new(fn_type));
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
      Some(TokenType::Fn) => {
        self.parser.advance();
        self.fn_declaration();
      }
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

  fn fn_declaration(&mut self) {
    // this *might* work without having to bring unsafe rust into it... fingers crossed!
    let global_idx = self.parse_variable(CompileError::Expected("function name"));
    self.mark_initialized();
    self.function(FunctionType::Function);
    self.define_variable(global_idx);
  }

  fn declaration(&mut self) {
    let global = self.parse_variable(CompileError::Expected("variable name"));

    if let Some(TokenType::Assign) = self.parser.current_type() {
      self.parser.advance();
      self.expression();
    } else {
      self.emit_opcode(OpCode::Tuple(0));
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
    // check if this is actually a unit
    if let Some(TokenType::RightParen) = self.parser.current_type() {
      self.parser.advance();
      self.emit_opcode(OpCode::Tuple(0));
      return;
    }

    self.expression();
    match self.parser.current_type() {
      Some(TokenType::Comma) => {
        self.parser.advance();
        self.tuple();
      }
      _ => {
        self.parser.consume(
          TokenType::RightParen,
          CompileError::Expected("')' after expression"),
        );
      }
    }
  }

  fn block(&mut self) {
    self.begin_scope();
    self.raw_block();
    self.end_scope();
  }

  fn raw_block(&mut self) {
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
      self.emit_opcode(OpCode::Tuple(0));
    }

    self.parser.consume(
      TokenType::RightBrace,
      CompileError::Expected("'}' after block"),
    );
  }

  fn function(&mut self, fn_type: FunctionType) {
    self.push_context(fn_type);
    if fn_type != FunctionType::TopLevel {
      self.current_context_mut().function.name = self.parser.previous().unwrap().lexeme.clone();
    }
    self.begin_scope();

    self.parser.consume(
      TokenType::LeftParen,
      CompileError::Expected("'(' after function name"),
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
    self.raw_block();
    self.emit_opcode(OpCode::Return);

    let context = self.pop_context().unwrap();
    if crate::DEBUG && (self.parser.error.is_none() || crate::SUPER_DEBUG) {
      disassemble_chunk(
        &context.function.chunk,
        if context.function.name.len() > 0 {
          &context.function.name
        } else {
          "[script]"
        },
      );
    }

    let idx = self.make_constant(Value::Function(Rc::new(context.function)));
    self.emit_opcode(OpCode::Closure(idx, context.upvalues));
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
    self.emit_opcode(OpCode::Tuple(0));
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
      Some(TokenType::False) => self.emit_opcode(OpCode::False),
      Some(TokenType::True) => self.emit_opcode(OpCode::True),
      _ => unreachable!(),
    }
  }

  fn variable(&mut self, can_assign: bool) {
    self.named_variable(self.parser.previous().unwrap().lexeme.clone(), can_assign)
  }

  fn tuple(&mut self) {
    // if we get here, we've already parsed one expression
    let mut count: u8 = 1;

    loop {
      if let Some(TokenType::RightParen) = self.parser.current_type() {
        // tuples can end in a comma-right parenthesis combo
        // this allows single-item tuples to exist
        self.parser.advance();
        break;
      }

      self.expression();
      count += 1;

      match self.parser.current_type() {
        Some(TokenType::Comma) => self.parser.advance(),
        Some(TokenType::RightParen) => {
          self.parser.advance();
          break;
        }
        _ => (),
      }
    }

    self.emit_opcode(OpCode::Tuple(count));
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

    // let mut count: usize = 0;
    for i in (0..self.locals().len()).rev() {
      if self.locals()[i].depth > self.scope_depth() {
        if self.locals()[i].is_captured {
          self.emit_opcode(OpCode::CloseUpvalue);
        } else {
          self.emit_opcode(OpCode::Pop);
        }
        self.locals_mut().remove(i);
        // count += 1;
      } else {
        break;
      }
    }

    /* todo: fix this so PopN isn't useless
    if count == 1 {
      self.emit_opcode(OpCode::Pop);
    } else if count > 1 {
      self.emit_opcode(OpCode::PopN(count));
    }
    */
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
    if self.scope_depth() != 0 {
      let idx = self.locals().len() - 1;
      self.locals_mut()[idx].depth = self.scope_depth();
    }
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
      let name = self.parser.previous().unwrap().lexeme.clone();
      let name_exists = self
        .locals()
        .iter()
        .rev()
        .filter(|local| local.depth != -1 && local.depth < self.scope_depth())
        .find(|local| &name == &local.name)
        .is_some();

      if name_exists {
        self
          .parser
          .set_error(CompileError::VariableAlreadyExists(name.clone()));
      } else {
        self.locals_mut().push(Local {
          name,
          depth: -1,
          is_captured: false,
        });
      }
    }
  }

  fn identifier_constant(&mut self, lexeme: String) -> usize {
    self.make_constant(Value::String(lexeme))
  }

  fn resolve_local(&mut self, name: &String, context_idx: usize) -> Option<usize> {
    let context = self.contexts.iter_mut().nth_back(context_idx);
    let context = match context {
      Some(c) => c,
      // we've run out of compile contexts
      None => return None,
    };

    match context.resolve_local(name) {
      Ok(local) => local,
      Err(err) => {
        self.parser.set_error(err);
        None
      }
    }
  }

  fn add_upvalue(&mut self, index: usize, is_local: bool, context_idx: usize) -> usize {
    // make sure we don't already have the upvalue before adding
    let upvalues = &mut self
      .contexts
      .iter_mut()
      .nth_back(context_idx)
      .unwrap()
      .upvalues;
    for i in 0..upvalues.len() {
      let upvalue = upvalues[i];
      if let Upvalue::Local(local_index) = upvalue {
        if local_index == index {
          return i;
        }
      }
    }

    upvalues.push(if is_local {
      Upvalue::Local(index)
    } else {
      Upvalue::Upvalue(index)
    });
    upvalues.len() - 1
  }

  fn resolve_upvalue(&mut self, name: &String, context_idx: usize) -> Option<usize> {
    if let Some(idx) = self.resolve_local(name, context_idx + 1) {
      self
        .contexts
        .iter_mut()
        .nth_back(context_idx + 1)
        .unwrap()
        .locals[idx]
        .is_captured = true;
      Some(self.add_upvalue(idx, true, context_idx))
    } else if context_idx + 1 >= self.contexts.len() {
      // if we continue here, we'd get stuck in an infinite loop until the stack overflows
      // this is because we are out of contexts to check
      None
    } else if let Some(idx) = self.resolve_upvalue(name, context_idx + 1) {
      Some(self.add_upvalue(idx, false, context_idx))
    } else {
      None
    }
  }

  fn named_variable(&mut self, lexeme: String, can_assign: bool) {
    let (get_op, set_op) = if let Some(idx) = self.resolve_local(&lexeme, 0) {
      (OpCode::GetLocal(idx), OpCode::SetLocal(idx))
    } else if let Some(idx) = self.resolve_upvalue(&lexeme, 0) {
      (OpCode::GetUpvalue(idx), OpCode::SetUpvalue(idx))
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
    self.emit_opcode(OpCode::Return);
    let context = self.contexts.pop().unwrap();

    if crate::DEBUG && (self.parser.error.is_none() || crate::SUPER_DEBUG) {
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
