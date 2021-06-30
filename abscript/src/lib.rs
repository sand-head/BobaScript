use compiler::CompileError;
use thiserror::Error;

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
  #[error("Compile error: {0}")]
  CompileError(#[from] CompileError),
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
