use bobascript::{value::Value, vm::VM};

mod common;

#[test]
fn concatenation() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let test = "1" + {
      let test2 = 15;
      test2 / 3
    };
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "test", Value::String("15".to_string()));
}

#[test]
fn empty() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    {}
    if true {}
    if false {} else {}
    "#,
  );
  assert!(result.is_ok());
}

#[test]
fn scope() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let result = true;
    let a = "outer";

    {
      let a = "inner";
      log a;
      result = result and a == "inner";
    }

    log a;
    result = result and a == "outer";
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "result", Value::Boolean(true));
}
