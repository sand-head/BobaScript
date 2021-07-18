use std::rc::Rc;

use bobascript_parser::{grammar::AstParser, Parser, SyntaxError};
use thiserror::Error;

use self::compiler::Compiler;
use crate::{
  chunk::{Chunk, Upvalue},
  value::Function,
};

mod compiler;
mod expressions;
mod statements;

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Error, Clone)]
pub enum CompileError {
  #[error("Undefined behavior: {0}")]
  UndefinedBehavior(String),
  #[error("Syntax error: {0}")]
  SyntaxError(#[from] SyntaxError),
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
  name: String,
  // todo: change this so we don't use -1 for uninitialized locals
  depth: i32,
  is_captured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionType {
  /// The root (or top) level script.
  TopLevel,
  /// A block expression.
  Block,
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
        name: "".to_string(),
        depth: 0,
        is_captured: false,
      }],
      upvalues: Vec::new(),
      scope_depth: 0,
    }
  }

  fn chunk_mut(&mut self) -> &mut Chunk {
    &mut self.function.chunk
  }

  fn resolve_local(&self, name: &str) -> CompileResult<Option<usize>> {
    for i in (0..self.locals.len()).rev() {
      if name == self.locals[i].name {
        return if self.locals[i].depth == -1 {
          Err(CompileError::VariableDoesNotExist(
            self.locals[i].name.clone(),
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
  let ast = AstParser::parse_ast(&source.into())?;
  let mut compiler = Compiler::new();
  compiler.compile(&ast)
}
