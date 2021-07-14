use bobascript::vm::{RuntimeError, VM};

mod common;

#[test]
fn tuple() {
  let mut vm = VM::default();
  assert_runtime_err!(
    vm,
    r#"
    #[]();
    "#,
    RuntimeError::InvalidCallSignature
  );
}

#[test]
fn record() {
  let mut vm = VM::default();
  assert_runtime_err!(
    vm,
    r#"
    #{}();
    "#,
    RuntimeError::InvalidCallSignature
  );
}

#[test]
fn number() {
  let mut vm = VM::default();
  assert_runtime_err!(
    vm,
    r#"
    (1 + 2)();
    "#,
    RuntimeError::InvalidCallSignature
  );
}

#[test]
fn bool() {
  let mut vm = VM::default();
  assert_runtime_err!(
    vm,
    r#"
    true();
    "#,
    RuntimeError::InvalidCallSignature
  );
}

#[test]
fn string() {
  let mut vm = VM::default();
  assert_runtime_err!(
    vm,
    r#"
    "totally a function I swear"();
    "#,
    RuntimeError::InvalidCallSignature
  );
}
