#[macro_export]
macro_rules! assert_eval {
  ($vm:expr, $eval:expr, $expected:expr) => {
    let function = bobascript::compiler::compile_expr($eval).unwrap();
    let result = $vm.evaluate(function);
    println!("eval result: {:?}", result);
    let value = result.unwrap();
    assert!(Value::equal(&value, &$expected));
  };
}

#[macro_export]
macro_rules! assert_runtime_err {
  ($result:expr, $expected:expr) => {
    assert!(
      if let Err(bobascript::InterpretError::RuntimeError(result)) = $result {
        std::mem::discriminant(&result) == std::mem::discriminant(&$expected)
      } else {
        false
      }
    );
  };
}

#[macro_export]
macro_rules! assert_compile_err {
  ($result:expr, $expected:expr) => {
    assert!(
      if let Err(bobascript::InterpretError::CompileError(result)) = $result {
        std::mem::discriminant(&result) == std::mem::discriminant(&$expected)
      } else {
        false
      }
    );
  };
}
