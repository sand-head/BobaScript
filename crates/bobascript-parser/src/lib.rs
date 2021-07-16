use ast::{Expr, Stmt};
use lalrpop_util::{lalrpop_mod, ParseError};
use thiserror::Error;

pub mod ast;
lalrpop_mod!(#[allow(clippy::all)] pub grammar);

#[derive(Debug, Error, Clone)]
pub enum SyntaxError {
  #[error("Unexpected character '{1}' on line {0}.")]
  UnexpectedCharacter(usize, char),
  #[error("Unterminated string on line {0}.")]
  UnterminatedString(usize),
}
type Result<T> = std::result::Result<T, SyntaxError>;

impl<T1, T2> From<ParseError<usize, T1, T2>> for SyntaxError {
  fn from(error: ParseError<usize, T1, T2>) -> Self {
    match error {
      ParseError::InvalidToken { location } => todo!(),
      ParseError::UnrecognizedEOF { location, expected } => todo!(),
      ParseError::UnrecognizedToken { token, expected } => todo!(),
      ParseError::ExtraToken { token } => todo!(),
      ParseError::User { error } => todo!(),
    }
  }
}

pub trait Parser<T> {
  fn parse_ast<'input>(input: &'input str) -> Result<T>;
}
impl Parser<Vec<Box<Stmt>>> for crate::grammar::StmtsParser {
  fn parse_ast<'input>(input: &'input str) -> Result<Vec<Box<Stmt>>> {
    let parser = crate::grammar::StmtsParser::new();
    let mut errors = Vec::new();
    let expr = parser.parse(&mut errors, input);

    if errors.is_empty() {
      Ok(expr.unwrap())
    } else {
      Err(errors.pop().unwrap().into())
    }
  }
}
impl Parser<Box<Expr>> for crate::grammar::ExprParser {
  fn parse_ast<'input>(input: &'input str) -> Result<Box<Expr>> {
    let parser = crate::grammar::ExprParser::new();
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
  use crate::{
    ast::Expr,
    grammar::{ExprParser, StmtsParser},
    Parser,
  };

  #[test]
  fn parse_function_stmt() {
    let stmt = StmtsParser::parse_ast("fn test() { 3 };").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"[Function("test", [], Block([], Some(Constant(Number(3.0)))))]"#
    );
    let stmt = StmtsParser::parse_ast("fn test(t1, t2, t3,) { 3 };").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"[Function("test", ["t1", "t2", "t3"], Block([], Some(Constant(Number(3.0)))))]"#
    );
  }

  #[test]
  fn parse_declaration_stmt() {
    let stmt = StmtsParser::parse_ast("const test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"[Const("test", Binary(Constant(Number(5.2)), Multiply, Constant(Number(3.0))))]"#
    );

    let stmt = StmtsParser::parse_ast("let test = 5.2 * 3;").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"[Let("test", Some(Binary(Constant(Number(5.2)), Multiply, Constant(Number(3.0)))))]"#
    );

    let stmt = StmtsParser::parse_ast("let test = 22.5 * if true {3} else {4};").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"[Let("test", Some(Binary(Constant(Number(22.5)), Multiply, If(Constant(True), Block([], Some(Constant(Number(3.0)))), Some(Block([], Some(Constant(Number(4.0)))))))))]"#
    );

    let stmt = StmtsParser::parse_ast("let test;").unwrap();
    assert_eq!(&format!("{:?}", stmt), r#"[Let("test", None)]"#);
  }

  #[test]
  fn parse_return_stmt() {
    let stmt = StmtsParser::parse_ast(r#"return "howdy!";"#).unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      r#"[Return(Some(Constant(String("\"howdy!\""))))]"#
    );
  }

  #[test]
  fn parse_expression_stmt() {
    let stmt = StmtsParser::parse_ast("22.5 * (44 + 66);").unwrap();
    assert_eq!(
      &format!("{:?}", stmt),
      "[Expression(Binary(Constant(Number(22.5)), Multiply, Binary(Constant(Number(44.0)), Add, Constant(Number(66.0)))))]"
    );
  }

  #[test]
  fn parse_block_expr() {
    let expr = ExprParser::parse_ast("{15 + 1; 3}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Block([Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))], Some(Constant(Number(3.0))))"
    );
  }

  #[test]
  fn parse_if_expr() {
    let expr = ExprParser::parse_ast(r#"if 3 == "3" {15 + 1; 3}"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"If(Binary(Constant(Number(3.0)), Equal, Constant(String("\"3\""))), Block([Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))], Some(Constant(Number(3.0)))), None)"#
    );

    let expr = ExprParser::parse_ast("if 3 {3} else if 6 {6}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "If(Constant(Number(3.0)), Block([], Some(Constant(Number(3.0)))), Some(If(Constant(Number(6.0)), Block([], Some(Constant(Number(6.0)))), None)))"
    );
  }

  #[test]
  fn parse_while_expr() {
    let expr = ExprParser::parse_ast("while true {15 + 1;}").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "While(Constant(True), [Expression(Binary(Constant(Number(15.0)), Add, Constant(Number(1.0))))])"
    );
  }

  #[test]
  fn parse_log_expr() {
    let expr = ExprParser::parse_ast(r#"log(a = "arg")"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Log(Assign(Constant(Ident("a")), Assign, Constant(String("\"arg\""))))"#
    );
  }

  #[test]
  fn parse_assign_expr() {
    let expr = ExprParser::parse_ast("a = 5").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Assign(Constant(Ident("a")), Assign, Constant(Number(5.0)))"#
    );
    let expr = ExprParser::parse_ast("a *= b = 5").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Assign(Constant(Ident("a")), MultiplyAssign, Assign(Constant(Ident("b")), Assign, Constant(Number(5.0))))"#
    );
  }

  #[test]
  fn parse_binary_expr() {
    let expr = ExprParser::parse_ast("22.5 * -44 + 66").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      "Binary(Binary(Constant(Number(22.5)), Multiply, Unary(Negate, Constant(Number(44.0)))), Add, Constant(Number(66.0)))"
    );
  }

  #[test]
  fn parse_call_expr() {
    let expr = ExprParser::parse_ast("test(3 * 5, 4,)").unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Call(Constant(Ident("test")), [Binary(Constant(Number(3.0)), Multiply, Constant(Number(5.0))), Constant(Number(4.0))])"#
    );
  }

  #[test]
  fn parse_complex_tuples() {
    let expr = ExprParser::parse_ast(r#"#[1, 3, 5, #["test", "I hope this works!!"]]"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Constant(Tuple([Constant(Number(1.0)), Constant(Number(3.0)), Constant(Number(5.0)), Constant(Tuple([Constant(String("\"test\"")), Constant(String("\"I hope this works!!\""))]))]))"#
    );

    let expr =
      ExprParser::parse_ast(r#"#[1, 3, 5, #["test", "I hope this works!!"]][3][1]"#).unwrap();
    assert_eq!(
      &format!("{:?}", expr),
      r#"Index(Index(Constant(Tuple([Constant(Number(1.0)), Constant(Number(3.0)), Constant(Number(5.0)), Constant(Tuple([Constant(String("\"test\"")), Constant(String("\"I hope this works!!\""))]))])), Constant(Number(3.0))), Constant(Number(1.0)))"#
    );
  }
}
