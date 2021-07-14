use bobascript::{compiler::compile, value::Value, vm::VM};

mod common;

#[test]
fn closures_with_open_upvalues_work() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    fn outer() {
      let x = "outside";
      fn inner() {
        x
      };
      inner()
    };
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "outer()", Value::String("outside".to_string()));
}

#[test]
fn closures_with_closed_upvalues_work() {
  let mut vm = VM::default();
  let function = compile(
    r#"
    fn outer() {
      let x = "outside";
      fn inner() {
        x
      };
      inner
    };

    let closure = outer();
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "closure()", Value::String("outside".to_string()));
}
