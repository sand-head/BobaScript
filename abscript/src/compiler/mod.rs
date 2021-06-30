use thiserror::Error;

use self::{
  parser::Parser,
  rules::{ParseRule, Precedence},
  scanner::TokenType,
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

#[derive(Debug, Error)]
pub enum CompileError {
  #[error("Unexpected character '{1}' on line {0}.")]
  UnexpectedCharacter(usize, char),
  #[error("Unterminated string on line {0}.")]
  UnterminatedString(usize),
  #[error("Expected {0}.")]
  Expected(&'static str),
}

pub struct Compiler<'a> {
  parser: Parser,
  chunk: &'a mut Chunk,
}
impl<'a> Compiler<'a> {
  pub fn new(chunk: &'a mut Chunk) -> Self {
    Self {
      parser: Parser::new(String::from("")),
      chunk,
    }
  }

  pub fn compile(&mut self, source: String) -> bool {
    self.parser = Parser::new(source);

    self.parser.advance();
    self.expression();
    self
      .parser
      .consume(TokenType::EOF, CompileError::Expected("end of expression"));
    self.end_compiler();

    !self.parser.had_error
  }

  fn parse_precedence(&mut self, precedence: Precedence) {
    self.parser.advance();
    let rule_prefix = get_rule(self.parser.previous.as_ref().unwrap().token_type).prefix;

    match rule_prefix {
      Some(prefix) => prefix(self),
      None => self
        .parser
        .print_error(CompileError::Expected("expression")),
    }

    while precedence <= get_rule(self.parser.current.as_ref().unwrap().token_type).precedence {
      self.parser.advance();
      let rule_infix = get_rule(self.parser.previous.as_ref().unwrap().token_type).infix;
      if let Some(infix) = rule_infix {
        infix(self);
      }
    }
  }

  fn expression(&mut self) {
    self.parse_precedence(Precedence::Assignment);
  }

  fn unary(&mut self) {
    let unary_operator = self.parser.previous.as_ref().unwrap().token_type.clone();

    // compile the operand
    self.parse_precedence(Precedence::Unary);

    // emit the operator
    match unary_operator {
      TokenType::Minus => self.emit_opcode(OpCode::Negate),
      _ => unreachable!(),
    }
  }

  fn binary(&mut self) {
    let binary_operator = self.parser.previous.as_ref().unwrap().token_type.clone();
    let rule: usize = get_rule(binary_operator).precedence.into();
    self.parse_precedence((rule + 1).into());

    match binary_operator {
      TokenType::Asterisk => self.emit_opcode(OpCode::Multiply),
      TokenType::Carrot => self.emit_opcode(OpCode::Exponent),
      TokenType::Minus => self.emit_opcode(OpCode::Subtract),
      TokenType::Plus => self.emit_opcode(OpCode::Add),
      TokenType::Slash => self.emit_opcode(OpCode::Divide),
      _ => unreachable!(),
    }
  }

  fn grouping(&mut self) {
    self.expression();
    self.parser.consume(
      TokenType::RightParen,
      CompileError::Expected("')' after expression"),
    );
  }

  fn number(&mut self) {
    let num = &self.parser.previous.as_ref().unwrap().lexeme;
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
      .write(opcode, self.parser.previous.as_ref().unwrap().line);
  }

  fn end_compiler(&mut self) {
    self.emit_opcode(OpCode::Return);
    if crate::DEBUG && !self.parser.had_error {
      disassemble_chunk(self.chunk, "code");
    }
  }
}

/// Gets the appropriate `ParseRule` for the given `TokenType`.
fn get_rule(token_type: TokenType) -> ParseRule {
  match token_type {
    TokenType::LeftParen => parse_prefix!(|c| c.grouping(), None),
    TokenType::Asterisk => parse_infix!(|c| c.binary(), Factor),
    TokenType::Carrot => parse_infix!(|c| c.binary(), Exponent),
    TokenType::Minus => parse_both!(|c| c.unary(), |c| c.binary(), Term),
    TokenType::Plus => parse_infix!(|c| c.binary(), Term),
    TokenType::Slash => parse_infix!(|c| c.binary(), Factor),
    TokenType::Number => parse_prefix!(|c| c.number(), None),
    _ => parse_none!(),
  }
}
