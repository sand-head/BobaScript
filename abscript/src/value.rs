use std::{convert::TryInto, fmt};

use crate::vm::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
  Number(f64),
  Boolean(bool),
}

impl TryInto<f64> for Value {
  type Error = RuntimeError;

  fn try_into(self) -> Result<f64, Self::Error> {
    if let Value::Number(num) = self {
      Ok(num)
    } else {
      Err(RuntimeError::TypeError {
        expected: "number",
        found: self,
      })
    }
  }
}
impl TryInto<bool> for Value {
  type Error = RuntimeError;

  fn try_into(self) -> Result<bool, Self::Error> {
    if let Value::Boolean(boolean) = self {
      Ok(boolean)
    } else {
      Err(RuntimeError::TypeError {
        expected: "boolean",
        found: self,
      })
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Number(num) => write!(f, "{}", num),
      Self::Boolean(boolean) => write!(f, "{}", boolean),
    }
  }
}
