use std::{cell::RefCell, convert::TryInto, fmt, rc::Rc};

use crate::{chunk::Chunk, vm::RuntimeError};

#[derive(Debug)]
pub struct Function {
  pub arity: u8,
  pub chunk: Chunk,
  pub name: String,
}
impl Default for Function {
  fn default() -> Self {
    Self {
      arity: 0,
      chunk: Chunk::default(),
      name: String::new(),
    }
  }
}

pub struct NativeFunction {
  pub function: fn(&[Value]) -> Result<Value, RuntimeError>,
}
impl fmt::Debug for NativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<native fn>")
  }
}

#[derive(Clone)]
pub struct Closure {
  pub function: Rc<Function>,
  pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}
impl fmt::Debug for Closure {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<closure {}>", self.function.name)
  }
}

#[derive(Debug, Clone)]
pub enum Upvalue {
  Open(usize),
  Closed(Value),
}

#[derive(Debug, Clone)]
pub enum Value {
  Tuple(Box<[Value]>),
  Number(f64),
  Boolean(bool),
  String(String),
  Function(Rc<Function>),
  NativeFunction(Rc<RefCell<NativeFunction>>),
  Closure(Closure),
}

impl Value {}
impl Value {
  pub fn get_unit() -> Value {
    Value::Tuple(vec![].into_boxed_slice())
  }
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
      Self::Tuple(tuple) => Ok(format!(
        "({})",
        tuple
          .iter()
          .map(|v| format!("{}", v))
          .collect::<Vec<String>>()
          .join(", ")
      )),
      Self::Number(num) => Ok(format!("{}", num)),
      Self::Boolean(bool) => Ok(format!("{}", bool)),
      Self::String(str) => Ok(str),
      Self::Function(function) if function.name.len() > 0 => Ok(format!("<fn {}>", function.name)),
      Self::NativeFunction(native_fn) => Ok(format!("{:?}", native_fn)),
      Self::Closure(closure) if closure.function.name.len() > 0 => {
        Ok(format!("<fn {}>", closure.function.name))
      }
      Self::Function(_) | Self::Closure(_) => Ok("<script>".to_string()),
    }
  }
}
impl TryInto<Rc<Function>> for Value {
  type Error = RuntimeError;

  fn try_into(self) -> Result<Rc<Function>, Self::Error> {
    match self {
      Value::Function(function) => Ok(function),
      Value::Closure(closure) => Ok(closure.function),
      _ => Err(RuntimeError::TypeError {
        expected: "function",
        found: self,
      }),
    }
  }
}
impl TryInto<Rc<RefCell<NativeFunction>>> for Value {
  type Error = RuntimeError;

  fn try_into(self) -> Result<Rc<RefCell<NativeFunction>>, Self::Error> {
    if let Value::NativeFunction(native_fn) = self {
      Ok(native_fn)
    } else {
      Err(RuntimeError::TypeError {
        expected: "function",
        found: self,
      })
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
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
