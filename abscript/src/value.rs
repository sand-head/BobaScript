use std::{convert::TryInto, fmt};

use crate::vm::RuntimeError;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
  Unit,
  Number(f64),
  Boolean(bool),
  String(String),
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
    if let Value::Boolean(bool) = self {
      Ok(bool)
    } else {
      Err(RuntimeError::TypeError {
        expected: "boolean",
        found: self,
      })
    }
  }
}
impl TryInto<String> for Value {
  type Error = RuntimeError;

  fn try_into(self) -> Result<String, Self::Error> {
    match self {
      Self::Number(num) => Ok(format!("{}", num)),
      Self::Boolean(bool) => Ok(format!("{}", bool)),
      Self::String(str) => Ok(str),
      _ => Err(RuntimeError::TypeError {
        expected: "string",
        found: self,
      }),
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Unit => write!(f, "()"),
      _ => {
        let value: String = Value::try_into(self.clone()).unwrap();
        if let Self::String(_) = self {
          write!(f, "\"{}\"", value)
        } else {
          write!(f, "{}", value)
        }
      }
    }
  }
}
