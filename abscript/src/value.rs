use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub enum Value {
  Number(f64),
}

impl Display for Value {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      Self::Number(num) => write!(f, "{}", num),
    }
  }
}