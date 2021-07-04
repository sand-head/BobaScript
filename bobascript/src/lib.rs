use compiler::CompileError;
use thiserror::Error;
use vm::RuntimeError;

pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod value;
pub mod vm;

#[cfg(not(feature = "debug"))]
pub const DEBUG: bool = false;
#[cfg(feature = "debug")]
pub const DEBUG: bool = true;

pub type InterpretResult<T> = Result<T, InterpretError>;

#[derive(Error, Debug)]
pub enum InterpretError {
  #[error("An unknown error has occurred.")]
  Unknown,
  #[error("An error occurred during compilation:\n{0}")]
  CompileError(#[from] CompileError),
  #[error("An error occurred during execution:\n{0}")]
  RuntimeError(#[from] RuntimeError),
}

#[cfg(test)]
mod tests {
  use crate::vm::VM;

  #[test]
  fn no_type_error_when_concat_string_and_block_expr() {
    let mut vm = VM::new();
    let result = vm.interpret("let test = \"1\" + { let test2 = 15; test2 / 3 };");
    assert!(result.is_ok());
  }
}
