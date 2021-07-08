use bobascript::{value::Value, vm::VM};

mod common;

#[test]
fn template() {
  let mut vm = VM::new();
}

#[test]
fn closures_with_open_upvalues_work() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    fn outer() {
      let x = "outside";
      fn inner() {
        x
      }
      inner()
    }
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "outer()", Value::String("outside".to_string()));
}

#[test]
fn closures_with_closed_upvalues_work() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    fn outer() {
      let x = "outside";
      fn inner() {
        x
      }
      inner
    }

    let closure = outer();
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "closure()", Value::String("outside".to_string()));
}
