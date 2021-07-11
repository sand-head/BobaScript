pub type Ast = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
  Function(String, Box<Expr>),
  Const(String, Box<Expr>),
  Let(String, Option<Box<Expr>>),
  Return(Option<Box<Expr>>),
  Expression(Box<Expr>),
}

#[derive(Debug)]
pub enum Expr {
  Block(Vec<Box<Stmt>>, Option<Box<Expr>>),
  Call(Box<Expr>, Vec<Box<Expr>>),
  Unary(UnaryOp, Box<Expr>),
  Binary(Box<Expr>, BinaryOp, Box<Expr>),
  If(
    /* condition: */ Box<Expr>,
    /* true branch: */ Box<Expr>,
    /* false branch: */ Option<Box<Expr>>,
  ),
  Ident(String),
  Number(f64),
  String(String),
}

#[derive(Debug)]
pub enum UnaryOp {
  Negate,
  Not,
}

#[derive(Debug)]
pub enum BinaryOp {
  Assign,
  AddAssign,
  SubtractAssign,
  MultiplyAssign,
  DivideAssign,
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
