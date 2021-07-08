use bobascript::{compiler::CompileError, value::Value, vm::VM};

mod common;

#[test]
fn body_must_be_block() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    fn f() 123;
    "#,
  );
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::Expected("block after parameters"));
}

#[test]
fn empty_body() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    fn f() {}
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "f()", Value::get_unit());
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
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "fib(10)", Value::Number(55.0));
}
