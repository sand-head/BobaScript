use lalrpop_util::lalrpop_mod;
use thiserror::Error;

pub mod ast;
lalrpop_mod!(pub grammar);

#[derive(Debug, Error, Clone)]
pub enum SyntaxError {
  #[error("Unexpected character '{1}' on line {0}.")]
  UnexpectedCharacter(usize, char),
  #[error("Unterminated string on line {0}.")]
  UnterminatedString(usize),
}
pub type Result<T> = std::result::Result<T, SyntaxError>;

mod tests {
  #![allow(unused_imports)]
  use crate::grammar::{ExprParser, StmtParser};

  #[test]
  fn parse_function_stmt() {
    let stmt = StmtParser::new().parse("fn test() { 3 };").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Function("test", [], Block([], Some(Constant(Number(3.0)))))"#
    );
    let stmt = StmtParser::new()
      .parse("fn test(t1, t2, t3,) { 3 };")
      .unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Function("test", ["t1", "t2", "t3"], Block([], Some(Constant(Number(3.0)))))"#
    );
  }

  #[test]
  fn parse_declaration_stmt() {
    let stmt = StmtParser::new().parse("const test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Const("test", Binary(Constant(Number(5.2)), Multiply, Constant(Number(3.0))))"#
    );

    let stmt = StmtParser::new().parse("let test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Let("test", Some(Binary(Constant(Number(5.2)), Multiply, Constant(Number(3.0)))))"#
    );

    let stmt = StmtParser::new().parse("let test;").unwrap();
    assert_eq!(&format!("{:?}", stmt), r#"Let("test", None)"#);
  }

  #[test]
  fn parse_return_stmt() {
    let stmt = StmtParser::new().parse(r#"return "howdy!";"#).unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Return(Some(Constant(String("\"howdy!\""))))"#
    );
  }

  #[test]
  fn parse_expression_stmt() {
    let stmt = StmtParser::new().parse("22.5 * (44 + 66);").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      "Expression(Binary(Constant(Number(22.5)), Multiply, Binary(Constant(Number(44.0)), Add, Constant(Number(66.0)))))"
    );
  }

  #[test]
  fn parse_block_expr() {
    let expr = ExprParser::new().parse("{15 + 1; 3}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Block([Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))], Some(Constant(Number(3.0))))"
    );
  }

  #[test]
  fn parse_if_expr() {
    let expr = ExprParser::new()
      .parse(r#"if 3 == "3" {15 + 1; 3}"#)
      .unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"If(Binary(Constant(Number(3.0)), Equal, Constant(String("\"3\""))), Block([Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))], Some(Constant(Number(3.0)))), None)"#
    );

    let expr = ExprParser::new().parse("if 3 {3} else if 6 {6}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "If(Constant(Number(3.0)), Block([], Some(Constant(Number(3.0)))), Some(Block([], Some(If(Constant(Number(6.0)), Block([], Some(Constant(Number(6.0)))), None)))))"
    );
  }

  #[test]
  fn parse_while_expr() {
    let expr = ExprParser::new().parse("while true {15 + 1;}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "While(Constant(True), [Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))])"
    );
  }

  #[test]
  fn parse_binary_expr() {
    let expr = ExprParser::new().parse("22.5 * -44 + 66").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Binary(Binary(Constant(Number(22.5)), Multiply, Unary(Negate, Constant(Number(44.0)))), Add, Constant(Number(66.0)))"
    );
  }

  #[test]
  fn parse_call_expr() {
    let expr = ExprParser::new().parse("test(3 * 5, 4,)").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Call(Constant(Ident("test")), [Binary(Constant(Number(3.0)), Multiply, Constant(Number(5.0))), Constant(Number(4.0))])"#
    );
  }
}
