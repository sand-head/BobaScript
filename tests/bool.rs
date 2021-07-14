use bobascript::{value::Value, vm::VM};

mod common;

#[test]
fn equality() {
  let mut vm = VM::default();
  assert_eval!(vm, "true == true", Value::Boolean(true));
  assert_eval!(vm, "true == false", Value::Boolean(false));
  assert_eval!(vm, "false == true", Value::Boolean(false));
  assert_eval!(vm, "false == false", Value::Boolean(true));

  assert_eval!(vm, "true == 1", Value::Boolean(false));
  assert_eval!(vm, "false == 0", Value::Boolean(false));
  assert_eval!(vm, r#"true == "true""#, Value::Boolean(false));
  assert_eval!(vm, r#"false == "false""#, Value::Boolean(false));
  assert_eval!(vm, r#"false == """#, Value::Boolean(false));

  assert_eval!(vm, "true != true", Value::Boolean(false));
  assert_eval!(vm, "true != false", Value::Boolean(true));
  assert_eval!(vm, "false != true", Value::Boolean(true));
  assert_eval!(vm, "false != false", Value::Boolean(false));

  assert_eval!(vm, "true != 1", Value::Boolean(true));
  assert_eval!(vm, "false != 0", Value::Boolean(true));
  assert_eval!(vm, r#"true != "true""#, Value::Boolean(true));
  assert_eval!(vm, r#"false != "false""#, Value::Boolean(true));
  assert_eval!(vm, r#"false != """#, Value::Boolean(true));
}

#[test]
fn not() {
  let mut vm = VM::default();
  assert_eval!(vm, "!true", Value::Boolean(false));
  assert_eval!(vm, "!false", Value::Boolean(true));
  assert_eval!(vm, "!!true", Value::Boolean(true));
}
