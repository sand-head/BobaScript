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

#[derive(Debug, Error, Clone, Copy)]
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

  pub fn compile(&mut self, source: String) -> CompileResult<()> {
    self.parser = Parser::new(source);

    self.parser.advance();
    self.expression();
    self
      .parser
      .consume(TokenType::EOF, CompileError::Expected("end of expression"));
    self.end_compiler();

    match &self.parser.error {
      None => Ok(()),
      Some(err) => Err(*err),
    }
  }

  fn parse_precedence(&mut self, precedence: Precedence) {
    self.parser.advance();
    let rule_prefix = get_rule(self.parser.previous.as_ref().unwrap().token_type).prefix;

    match rule_prefix {
      Some(prefix) => prefix(self),
      None => self.parser.set_error(CompileError::Expected("expression")),
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
    let unary_operator = self.parser.previous.as_ref().unwrap().token_type;

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
    let binary_operator = self.parser.previous.as_ref().unwrap().token_type;
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
    let previous_type = self.parser.previous.as_ref().unwrap().token_type;
    match previous_type {
      TokenType::False => self.emit_opcode(OpCode::False),
      TokenType::True => self.emit_opcode(OpCode::True),
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

  fn string(&mut self) {
    let string = self.parser.previous.as_ref().unwrap().lexeme.clone();
    // strip the leading and trailing quotation mark off the lexeme:
    let string = string[1..(string.len() - 1)].to_string();
    let string_idx = self.make_constant(Value::String(string));
    self.emit_opcode(OpCode::Constant(string_idx))
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
    if crate::DEBUG && self.parser.error.is_none() {
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

    TokenType::Not => parse_prefix!(|c| c.unary(), None),
    TokenType::NotEqual => parse_infix!(|c| c.binary(), Equality),
    TokenType::Equal => parse_infix!(|c| c.binary(), Equality),
    TokenType::GreaterThan => parse_infix!(|c| c.binary(), Comparison),
    TokenType::GreaterEqual => parse_infix!(|c| c.binary(), Comparison),
    TokenType::LessThan => parse_infix!(|c| c.binary(), Comparison),
    TokenType::LessEqual => parse_infix!(|c| c.binary(), Comparison),

    TokenType::String => parse_prefix!(|c| c.string(), None),
    TokenType::Number => parse_prefix!(|c| c.number(), None),

    TokenType::False => parse_prefix!(|c| c.literal(), None),
    TokenType::True => parse_prefix!(|c| c.literal(), None),

    _ => parse_none!(),
  }
}
