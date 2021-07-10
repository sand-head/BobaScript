pub type Ast = Vec<Statement>;

pub enum Statement {
  Function(),
  Const(),
  Let(),
  Return(Option<Expression>),
  Expression(Expression),
}

pub enum Expression {
  Block(Vec<Statement>),
  Grouping(Box<Expression>),
  Call(/* todo: function */),
  UnaryOperation(UnaryOperator, Box<Expression>),
  BinaryOperation(Box<Expression>, UnaryOperator, Box<Expression>),
  If {
    condition: Box<Expression>,
    true_branch: Box<Expression>,
    false_branch: Option<Box<Expression>>,
  },
}

pub enum UnaryOperator {
  Minus,
  Not,
}

pub enum BinaryOperator {
  Asterisk,
  Carrot,
  Minus,
  Plus,
  Slash,
  NotEqual,
  Equal,
  GreaterThan,
  GreaterEqual,
  LessThan,
  LessEqual,
}
