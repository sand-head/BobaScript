use bobascript::{
  compiler::{compile, CompileError},
  value::Value,
  vm::{RuntimeError, VM},
};

mod common;

#[test]
fn associativity() {
  let mut vm = VM::new();
  let function = compile(
    r#"
    let a = "a";
    let b = "b";
    let c = "c";

    a = b = c;
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);

  println!("result: {:?}", result);
  assert!(result.is_ok());

  assert_eval!(vm, "a", Value::String("c".to_string()));
  assert_eval!(vm, "b", Value::String("c".to_string()));
  assert_eval!(vm, "c", Value::String("c".to_string()));
}

#[test]
fn global() {
  let mut vm = VM::new();
  let function = compile(r#"let a = "before";"#).unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("before".to_string()));

  let function = compile(r#"a = "after";"#).unwrap();
  let result = vm.interpret(function);
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("after".to_string()));

  assert_eval!(vm, r#"a = "arg""#, Value::String("arg".to_string()));
}

// actually, I decided this is weird but ok for now,
// since lalrpop gives us groupings essentially for free
// go buck wild with 'em for all I care
// #[test]
// fn grouping() {
//   let result = compile(
//     r#"
//     let a = "a";
//     (a) = "value";
//     "#,
//   );
//   println!("result: {:?}", result);
//   assert!(result.is_err());
//   assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
// }

#[test]
fn infix_operator() {
  let result = compile(
    r#"
    let a = "a";
    let b = "b";
    a + b = "value";
    "#,
  );
  println!("result: {:?}", result);
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
}

#[test]
fn local() {
  let mut vm = VM::new();
  let function = compile(
    r#"
    {
      let a = "before";
      log(a);

      a = "after";
      log(a);

      log(a = "arg");
      log(a);
    };
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  println!("result: {:?}", result);
  assert!(result.is_ok());
}

#[test]
fn prefix_operator() {
  let result = compile(
    r#"
    let a = "a";
    !a = "value";
    "#,
  );
  println!("result: {:?}", result);
  assert!(result.is_err());
  assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
}

#[test]
fn syntax() {
  let mut vm = VM::new();
  let function = compile(
    r#"
    let a = "before";
    let c = a = "var";
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  println!("result: {:?}", result);
  assert!(result.is_ok());
  assert_eval!(vm, "a", Value::String("var".to_string()));
  assert_eval!(vm, "c", Value::String("var".to_string()));
}

// we don't have classes yet, so why test for them?
// #[test]
// fn to_this() {
//   let result = compile(
//     r#"
//     class Foo {
//       Foo() {
//         this = "value";
//       };
//     };

//     Foo();
//     "#,
//   );
//   println!("result: {:?}", result);
//   assert!(result.is_err());
//   assert_compile_err!(result, CompileError::InvalidAssignmentTarget);
// }

#[test]
fn undefined() {
  let mut vm = VM::new();
  let function = compile(
    r#"
    unknown = "what";
    "#,
  )
  .unwrap();
  let result = vm.interpret(function);
  println!("result: {:?}", result);
  assert!(result.is_err());
  assert_runtime_err!(
    result,
    RuntimeError::UndefinedVariable("unknown".to_string())
  );
}
