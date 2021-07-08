use bobascript::{
  compiler::CompileError,
  value::Value,
  vm::{RuntimeError, VM},
};

mod common;

#[test]
fn associativity() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let a = "a";
    let b = "b";
    let c = "c";

    a = b = c;
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("c".to_string()));
  assert_eval!(vm, "b", Value::String("c".to_string()));
  assert_eval!(vm, "c", Value::String("c".to_string()));
}

#[test]
fn global() {
  let mut vm = VM::new();
  let result = vm.interpret(r#"let a = "before";"#);
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("before".to_string()));

  let result = vm.interpret(r#"a = "after";"#);
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("after".to_string()));

  assert_eval!(vm, r#"a = "arg""#, Value::String("arg".to_string()));
}

#[test]
fn grouping() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let a = "a";
    (a) = "value";
    "#,
  );
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
}

#[test]
fn infix_operator() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let a = "a";
    let b = "b";
    a + b = "value";
    "#,
  );
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
}

#[test]
fn local() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    {
      let a = "before";
      log a;

      a = "after";
      log a;

      log a = "arg";
      log a;
    }
    "#,
  );
  assert!(result.is_ok());
}

#[test]
fn prefix_operator() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let a = "a";
    !a = "value";
    "#,
  );
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
}

#[test]
fn syntax() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    let a = "before";
    let c = a = "var";
    "#,
  );
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("var".to_string()));
  assert_eval!(vm, "c", Value::String("var".to_string()));
}

#[test]
fn to_this() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    class Foo {
      Foo() {
        this = "value";
      }
    }

    Foo();
    "#,
  );
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
}

#[test]
fn undefined() {
  let mut vm = VM::new();
  let result = vm.interpret(
    r#"
    unknown = "what";
    "#,
  );
  assert!(result.is_err());
  assert_runtime_err!(
    result,
    RuntimeError::UndefinedVariable("unknown".to_string())
  );
}
