use std::collections::HashMap;

pub type Ast = Vec<Box<Stmt>>;

#[derive(Debug)]
pub enum Stmt {
  Function(String, Vec<String>, Box<Expr>),
  Const(String, Box<Expr>),
  Let(String, Option<Box<Expr>>),
  Return(Option<Box<Expr>>),
  Break(Option<Box<Expr>>),
  Expression(Box<Expr>),
}

#[derive(Debug)]
pub enum Expr {
  Error,
  /// Outputs the value of the contained [Expr] as a log.
  Log(Box<Expr>),
  Block(Vec<Box<Stmt>>, Option<Box<Expr>>),
  /// The first [Expr] is the condition, the second the "true" block,
  /// and the third the "false" block.
  ///
  /// Else-if statements are collapsed as if a standard if statement
  /// was put inside of the else block.
  If(
    /* condition: */ Box<Expr>,
    /* true branch: */ Box<Expr>,
    /* false branch: */ Option<Box<Expr>>,
  ),
  /// While [Expr] is true, do [Stmt]s.
  While(Box<Expr>, Vec<Box<Stmt>>),
  Assign(Box<Expr>, AssignOp, Box<Expr>),
  Binary(Box<Expr>, BinaryOp, Box<Expr>),
  Unary(UnaryOp, Box<Expr>),
  Property(Box<Expr>, String),
  Index(Box<Expr>, Box<Expr>),
  Call(Box<Expr>, Vec<Box<Expr>>),
  Constant(Constant),
}

#[derive(Debug)]
pub enum Constant {
  True,
  False,
  Ident(String),
  Number(f64),
  String(String),
  Tuple(Vec<Box<Expr>>),
  Record(HashMap<String, Box<Expr>>),
}

#[derive(Debug)]
pub enum UnaryOp {
  Negate,
  Not,
}

#[derive(Debug)]
pub enum BinaryOp {
  Or,
  And,
  Equal,
  NotEqual,
  GreaterThan,
  GreaterEqual,
  LessThan,
  LessEqual,
  Add,
  Subtract,
  Multiply,
  Divide,
  Exponent,
}

#[derive(Debug)]
pub enum AssignOp {
  Assign,
  AddAssign,
  SubtractAssign,
  MultiplyAssign,
  DivideAssign,
  ExponentAssign,
  OrAssign,
  AndAssign,
}
