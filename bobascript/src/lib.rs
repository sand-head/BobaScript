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

#[cfg(not(feature = "super_debug"))]
pub const SUPER_DEBUG: bool = false;
#[cfg(feature = "super_debug")]
pub const SUPER_DEBUG: bool = true;

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
