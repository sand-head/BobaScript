use std::{convert::From, fmt::Display, string::String};

use ast::Ast;
use lalrpop_util::{lalrpop_mod, ParseError};
use thiserror::Error;

pub mod ast;
lalrpop_mod!(#[allow(clippy::all)] pub grammar);

#[derive(Debug, Error, Clone)]
pub enum SyntaxError {
  #[error("{0}")]
  Generic(String),
  #[error("Expected {0}.")]
  Expected(String),
  #[error("Expected {0}; found token '{1}'.")]
  UnexpectedToken(String, String),
  #[error("Found extra token {0}.")]
  ExtraToken(String),
  #[error("Invalid token.")]
  Invalid,
}
type Result<T> = std::result::Result<T, SyntaxError>;

impl<T1, T2> From<ParseError<usize, T1, T2>> for SyntaxError
where
  T1: Display,
  T2: Into<String>,
{
  fn from(error: ParseError<usize, T1, T2>) -> Self {
    match error {
      ParseError::InvalidToken { location: _ } => SyntaxError::Invalid,
      ParseError::UnrecognizedEOF {
        location: _,
        expected,
      } => SyntaxError::Expected(expected.join(", ")),
      ParseError::UnrecognizedToken { token, expected } => {
        SyntaxError::UnexpectedToken(expected.join(", "), token.1.to_string())
      }
      ParseError::ExtraToken { token } => SyntaxError::ExtraToken(token.1.to_string()),
      ParseError::User { error } => SyntaxError::Generic(error.into()),
    }
  }
}

pub trait Parser<T> {
  fn parse_ast(input: &'_ str) -> Result<T>;
}
impl Parser<Ast> for crate::grammar::AstParser {
  fn parse_ast(input: &'_ str) -> Result<Ast> {
    let parser = crate::grammar::AstParser::new();
    let mut errors = Vec::new();
    let expr = parser.parse(&mut errors, input);

    if errors.is_empty() {
      Ok(expr.unwrap())
    } else {
      Err(errors.pop().unwrap().into())
    }
  }
}

mod tests {
  #![allow(unused_imports)]
  use crate::{ast::Expr, grammar::AstParser, Parser};

  #[test]
  fn parse_function_stmt() {
    let stmt = AstParser::parse_ast("fn test() { 3 };").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Ast([Function("test", [], Block([], Some(Constant(Number(3.0)))))], None)"#
    );
    let stmt = AstParser::parse_ast("fn test(t1, t2, t3,) { 3 };").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Ast([Function("test", ["t1", "t2", "t3"], Block([], Some(Constant(Number(3.0)))))], None)"#
    );
  }

  #[test]
  fn parse_declaration_stmt() {
    let stmt = AstParser::parse_ast("const test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Ast([Const("test", Binary(Constant(Number(5.2)), Multiply, Constant(Number(3.0))))], None)"#
    );

    let stmt = AstParser::parse_ast("let test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Ast([Let("test", Some(Binary(Constant(Number(5.2)), Multiply, Constant(Number(3.0)))))], None)"#
    );

    let stmt = AstParser::parse_ast("let test = 22.5 * if true {3} else {4};").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Ast([Let("test", Some(Binary(Constant(Number(22.5)), Multiply, If(Constant(True), Block([], Some(Constant(Number(3.0)))), Some(Block([], Some(Constant(Number(4.0)))))))))], None)"#
    );

    let stmt = AstParser::parse_ast("let test;").unwrap();
    assert_eq!(&format!("{:?}", stmt), r#"Ast([Let("test", None)], None)"#);
  }

  #[test]
  fn parse_return_stmt() {
    let stmt = AstParser::parse_ast(r#"return "howdy!";"#).unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"Ast([Return(Some(Constant(String("\"howdy!\""))))], None)"#
    );
  }

  #[test]
  fn parse_expression_stmt() {
    let stmt = AstParser::parse_ast("22.5 * (44 + 66);").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      "Ast([Expression(Binary(Constant(Number(22.5)), Multiply, Binary(Constant(Number(44.0)), Add, Constant(Number(66.0)))))], None)"
    );
  }

  #[test]
  fn parse_block_expr() {
    let expr = AstParser::parse_ast("{15 + 1; 3}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Ast([], Some(Block([Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))], Some(Constant(Number(3.0))))))"
    );
  }

  #[test]
  fn parse_if_expr() {
    let expr = AstParser::parse_ast(r#"if 3 == "3" {15 + 1; 3}"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(If(Binary(Constant(Number(3.0)), Equal, Constant(String("\"3\""))), Block([Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))], Some(Constant(Number(3.0)))), None)))"#
    );

    let expr = AstParser::parse_ast("if 3 {3} else if 6 {6}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Ast([], Some(If(Constant(Number(3.0)), Block([], Some(Constant(Number(3.0)))), Some(If(Constant(Number(6.0)), Block([], Some(Constant(Number(6.0)))), None)))))"
    );
  }

  #[test]
  fn parse_while_expr() {
    let expr = AstParser::parse_ast("while true {15 + 1;}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Ast([], Some(While(Constant(True), [Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))])))"
    );
  }

  #[test]
  fn parse_log_expr() {
    let expr = AstParser::parse_ast(r#"log(a = "arg")"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(Log(Assign(Constant(Ident("a")), Assign, Constant(String("\"arg\""))))))"#
    );
  }

  #[test]
  fn parse_assign_expr() {
    let expr = AstParser::parse_ast("a = 5").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(Assign(Constant(Ident("a")), Assign, Constant(Number(5.0)))))"#
    );
    let expr = AstParser::parse_ast("a *= b = 5").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(Assign(Constant(Ident("a")), MultiplyAssign, Assign(Constant(Ident("b")), Assign, Constant(Number(5.0))))))"#
    );
  }

  #[test]
  fn parse_binary_expr() {
    let expr = AstParser::parse_ast("22.5 * -44 + 66").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Ast([], Some(Binary(Binary(Constant(Number(22.5)), Multiply, Unary(Negate, Constant(Number(44.0)))), Add, Constant(Number(66.0)))))"
    );
  }

  #[test]
  fn parse_call_expr() {
    let expr = AstParser::parse_ast("test(3 * 5, 4,)").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(Call(Constant(Ident("test")), [Binary(Constant(Number(3.0)), Multiply, Constant(Number(5.0))), Constant(Number(4.0))])))"#
    );
  }

  #[test]
  fn parse_complex_tuples() {
    let expr = AstParser::parse_ast(r#"#[1, 3, 5, #["test", "I hope this works!!"]]"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(Constant(Tuple([Constant(Number(1.0)), Constant(Number(3.0)), Constant(Number(5.0)), Constant(Tuple([Constant(String("\"test\"")), Constant(String("\"I hope this works!!\""))]))]))))"#
    );

    let expr =
      AstParser::parse_ast(r#"#[1, 3, 5, #["test", "I hope this works!!"]][3][1]"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Ast([], Some(Index(Index(Constant(Tuple([Constant(Number(1.0)), Constant(Number(3.0)), Constant(Number(5.0)), Constant(Tuple([Constant(String("\"test\"")), Constant(String("\"I hope this works!!\""))]))])), Constant(Number(3.0))), Constant(Number(1.0)))))"#
    );
  }
}
