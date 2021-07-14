use bobascript::{compiler::compile, value::Value, vm::VM};

mod common;

#[test]
fn concatenation() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    let test = "1" + {
      let test2 = 15;
      test2 / 3
    };
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "test", Value::String("15".to_string()));
}

#[test]
fn empty() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    {};
    if true {};
    if false {} else {};
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
}

#[test]
fn scope() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    let result = true;
    let a = "outer";

    {
      let a = "inner";
      log(a);
      result &&= a == "inner";
    };

    log(a);
    result &&= a == "outer";
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "result", Value::Boolean(true));
}
