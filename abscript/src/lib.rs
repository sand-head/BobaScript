use scanner::ScanError;
use thiserror::Error;

pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod scanner;
pub mod value;
pub mod vm;

pub type InterpretResult<T> = Result<T, InterpretError>;

#[derive(Error, Debug)]
pub enum InterpretError {
  #[error("An error occurred while scanning the source code: {0}")]
  ScanError(#[from] ScanError),
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
