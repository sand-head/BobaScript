extern crate pest;
#[macro_use]
extern crate pest_derive;

use thiserror::Error;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod tokens;

#[derive(Debug, Error, Clone)]
pub enum SyntaxError {
  #[error("Unexpected character '{1}' on line {0}.")]
  UnexpectedCharacter(usize, char),
  #[error("Unterminated string on line {0}.")]
  UnterminatedString(usize),
}
pub type Result<T> = std::result::Result<T, SyntaxError>;

#[derive(Parser)]
#[grammar = "bobascript.pest"]
pub struct BobaScriptParser {}
