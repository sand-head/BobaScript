use std::{cell::RefCell, convert::TryInto, rc::Rc};

use bobascript::{
  compiler::compile,
  value::{NativeFunction, Value},
  vm::{RuntimeError, VM},
};

mod common;

#[test]
fn body_must_be_block() {
  let result = compile(
    r#"
    fn f() 123;
    "#,
  );
  assert!(result.is_err());
  // assert_compile_err!(result, CompileError::Expected("block after parameters"));
}

#[test]
fn empty_body() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    fn f() {};
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "f()", Value::get_unit());
}

#[test]
fn recursion_works() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    fn fib(n) {
      if n < 2 {
        n
      } else {
        fib(n - 2) + fib(n - 1)
      }
    };
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "fib(10)", Value::Number(55.0));
}

#[test]
fn native_fns_work() {
  fn test(params: &[Value]) -> Result<Value, RuntimeError> {
    println!("params: {:?}", params);
    if params.len() != 2 {
      return Err(RuntimeError::IncorrectParameterCount(
        2,
        params.len().try_into().unwrap(),
      ));
    }

    let a = params.get(0).unwrap();
    let b = params.get(1).unwrap();
    Ok(Value::String(format!("{}{}", a, b)))
  }

  let mut vm = VM::default();
  vm.define_native(
    "test".to_owned(),
    Rc::new(RefCell::new(NativeFunction { function: test })),
  );
  assert_eval!(vm, "test(10, 20)", Value::String("1020".to_string()));
}
