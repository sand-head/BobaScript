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

#[cfg(test)]
mod tests {
  use crate::vm::VM;

  #[test]
  fn no_type_error_when_concat_string_and_block_expr() {
    let mut vm = VM::new();
    let result = vm.interpret("let test = \"1\" + { let test2 = 15; test2 / 3 };");
    assert!(result.is_ok());
  }

  #[test]
  fn recursion_works() {
    let mut vm = VM::new();
    let result = vm.interpret(
      r#"
      fn fib(n) {
        if n < 2 {
          n
        } else {
          fib(n - 2) + fib(n - 1)
        }
      }

      log fib(10);
      "#,
    );
    println!("{:?}", result);
    assert!(result.is_ok());
    // let value = result.unwrap();
    // assert_eq!(value, Value::Number(55));
  }

  #[test]
  fn closures_with_open_upvalues_work() {
    let mut vm = VM::new();

    let result = vm.interpret(
      r#"
      fn outer() {
        let x = "outside";
        fn inner() {
          log x;
        }
        inner();
      }

      outer();
      "#,
    );

    println!("{:?}", result);
    assert!(result.is_ok());
    // let value = result.unwrap();
    // assert_eq!(value, Value::get_unit());
  }

  #[test]
  fn closures_with_closed_upvalues_work() {
    let mut vm = VM::new();

    let result = vm.interpret(
      r#"
      fn outer() {
        let x = "outside";
        fn inner() {
          log x;
        }
        inner
      }

      let closure = outer();
      closure();
      "#,
    );

    println!("{:?}", result);
    assert!(result.is_ok());
    // let value = result.unwrap();
    // assert_eq!(value, Value::get_unit());
  }
}