use bobascript::compiler::compile;

mod common;

#[test]
fn single_line() {
  let result = compile("// single-line comment");
  assert!(result.is_ok());
}

#[test]
fn multi_line() {
  let result = compile(
    r#"
    /*
    this is a multi-line comment
    */
    "#,
  );
  assert!(result.is_ok());
}
