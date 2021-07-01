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
  #[error("Compile error: {0}")]
  CompileError(#[from] CompileError),
  #[error("Runtime error: {0}")]
  RuntimeError(#[from] RuntimeError),
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
