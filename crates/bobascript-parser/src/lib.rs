use lalrpop_util::lalrpop_mod;
use thiserror::Error;

pub mod ast;
lalrpop_mod!(pub bobascript);
pub mod lexer;
pub mod parser;
pub mod tokens;

#[derive(Debug, Error, Clone)]
pub enum SyntaxError {
  #[error("Unexpected character '{1}' on line {0}.")]
  UnexpectedCharacter(usize, char),
  #[error("Unterminated string on line {0}.")]
  UnterminatedString(usize),
}
pub type Result<T> = std::result::Result<T, SyntaxError>;

mod tests {
  use crate::bobascript::{ExprParser, StmtParser};

  #[test]
  fn parse_function_stmt() {
    let stmt = StmtParser::new().parse("fn test() { 3 }").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Function("test", Block([], Some(Number(3.0))))"#
    );
  }

  #[test]
  fn parse_declaration_stmt() {
    let stmt = StmtParser::new().parse("const test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Const("test", Binary(Number(5.2), Multiply, Number(3.0)))"#
    );

    let stmt = StmtParser::new().parse("let test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Let("test", Some(Binary(Number(5.2), Multiply, Number(3.0))))"#
    );

    let stmt = StmtParser::new().parse("let test;").unwrap();
    assert_eq!(&format!("{:?}", stmt), r#"Let("test", None)"#);
  }

  #[test]
  fn parse_return_stmt() {
    let stmt = StmtParser::new().parse(r#"return "howdy!";"#).unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Return(Some(String("\"howdy!\"")))"#
    );
  }

  #[test]
  fn parse_expression_stmt() {
    let stmt = StmtParser::new().parse("22.5 * (44 + 66);").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      "Expression(Binary(Number(22.5), Multiply, Binary(Number(44.0), Add, Number(66.0))))"
    );
  }

  #[test]
  fn parse_block_expr() {
    let expr = ExprParser::new().parse("{15 + 1; 3}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Block([Expression(Binary(Number(15.0), Add, Number(1.0)))], Some(Number(3.0)))"
    );
  }

  #[test]
  fn parse_if_expr() {
    let expr = ExprParser::new()
      .parse(r#"if 3 == "3" {15 + 1; 3}"#)
      .unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"If(Binary(Number(3.0), Equal, String("\"3\"")), Block([Expression(Binary(Number(15.0), Add, Number(1.0)))], Some(Number(3.0))), None)"#
    );

    let expr = ExprParser::new().parse("if 3 {3} else if 6 {6}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "If(Number(3.0), Block([], Some(Number(3.0))), Some(Block([], Some(If(Number(6.0), Block([], Some(Number(6.0))), None)))))"
    );
  }

  #[test]
  fn parse_binary_expr() {
    let expr = ExprParser::new().parse("22.5 * 44 + 66").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Binary(Binary(Number(22.5), Multiply, Number(44.0)), Add, Number(66.0))"
    );
  }
}
