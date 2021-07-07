use std::rc::Rc;

use thiserror::Error;

use self::{
  compiler::Compiler,
  scanner::{Token, TokenType},
};
use crate::{chunk::Upvalue, value::Function};

mod compiler;
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

pub struct Local {
  name: Token,
  // todo: change this so we don't use -1 for uninitialized locals
  depth: i32,
  is_captured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
  upvalues: Vec<Upvalue>,
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
        is_captured: false,
      }],
      upvalues: Vec::new(),
      scope_depth: 0,
    }
  }

  fn resolve_local(&self, name: &String) -> CompileResult<Option<usize>> {
    for i in (0..self.locals.len()).rev() {
      if name == &self.locals[i].name.lexeme {
        return if self.locals[i].depth == -1 {
          Err(CompileError::VariableDoesNotExist(
            self.locals[i].name.lexeme.clone(),
          ))
        } else {
          // println!("yeah we got a {} at index {}", name, i);
          Ok(Some(i))
        };
      }
    }
    Ok(None)
  }
}

/// Compiles the given source code and returns its resulting function.
pub fn compile<S>(source: S) -> CompileResult<Rc<Function>>
where
  S: Into<String>,
{
  let mut compiler = Compiler::new(source);
  compiler.compile()
}

/// Compiles a single expression and returns its resulting function.
pub fn compile_expr<S>(expression: S) -> CompileResult<Rc<Function>>
where
  S: Into<String>,
{
  let mut compiler = Compiler::new(expression);
  compiler.compile_expr()
}
