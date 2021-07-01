use std::convert::TryInto;

use thiserror::Error;

use crate::{
  chunk::{Chunk, OpCode},
  compiler::Compiler,
  debug::disassemble_instruction,
  value::Value,
  InterpretResult,
};

macro_rules! binary_op {
  ($compiler:expr, $value_type:ident, $op:expr) => {{
    let b = $compiler.peek_and_pop_as::<$value_type>();
    let a = $compiler.peek_and_pop_as::<$value_type>();

    a.and_then(|a| b.and_then(|b| Ok($op(a, b))))
  }};
}

#[derive(Debug, Error)]
pub enum RuntimeError {
  #[error("An unknown error has occurred.")]
  Unknown,
  #[error("Type error: expected value of type \"{expected}\", found {found}.")]
  TypeError {
    expected: &'static str,
    found: Value,
  },
  #[error("The attempted operation is not supported.")]
  OperationNotSupported,
}

pub struct VM {
  chunk: Chunk,
  ip: usize,
  stack: Vec<Value>,
}
impl VM {
  pub fn new() -> Self {
    Self {
      chunk: Chunk::default(),
      ip: 0,
      stack: Vec::with_capacity(256),
    }
  }

  pub fn interpret<S>(&mut self, source: S) -> InterpretResult<Value>
  where
    S: Into<String>,
  {
    let mut chunk = Chunk::default();
    let mut compiler = Compiler::new(&mut chunk);
    match compiler.compile(source.into()) {
      Ok(_) => {
        self.chunk = chunk;
        self.ip = 0;
        self.run()
      }
      Err(err) => {
        self.stack.clear();
        Err(err.into())
      }
    }
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value)
  }

  fn peek(&self, n: usize) -> Option<&Value> {
    self.stack.iter().nth_back(n)
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }

  fn peek_and_pop_as<T>(&mut self) -> InterpretResult<T>
  where
    Value: TryInto<T, Error = RuntimeError>,
  {
    let value: T = self
      .peek(0)
      .ok_or_else(|| RuntimeError::Unknown)?
      .to_owned()
      .try_into()?;
    self.pop();
    Ok(value)
  }

  fn values_equal(&self, a: Value, b: Value) -> bool {
    match (a, b) {
      (Value::Number(a), Value::Number(b)) => a == b,
      (Value::Boolean(a), Value::Boolean(b)) => a == b,
      (Value::String(a), Value::String(b)) => a == b,
      _ => false,
    }
  }

  fn run(&mut self) -> InterpretResult<Value> {
    loop {
      let (instruction, line) = {
        let instruction = &self.chunk.code[self.ip];
        self.ip += 1;
        instruction
      };

      if crate::DEBUG {
        print!("\t");
        for value in self.stack.iter() {
          print!("[{}]", value);
        }
        println!();
        disassemble_instruction(&self.chunk, instruction, line, self.ip);
      }

      match instruction {
        OpCode::Constant(idx) => {
          let constant = self.chunk.constants[*idx].clone();
          self.push(constant);
        }
        OpCode::True => self.push(Value::Boolean(true)),
        OpCode::False => self.push(Value::Boolean(false)),
        OpCode::Equal => {
          let b = self.pop();
          let a = self.pop();
          let value = self.values_equal(a, b);
          self.push(Value::Boolean(value));
        }
        OpCode::GreaterThan => {
          let value = binary_op!(self, f64, |a, b| a > b)?;
          self.push(Value::Boolean(value));
        }
        OpCode::LessThan => {
          let value = binary_op!(self, f64, |a, b| a < b)?;
          self.push(Value::Boolean(value));
        }
        OpCode::Add => {
          let b = self.peek(0).ok_or_else(|| RuntimeError::Unknown)?.clone();
          let a = self.peek(1).ok_or_else(|| RuntimeError::Unknown)?.clone();

          match (a, b) {
            (Value::Number(_), Value::Number(_)) => {
              let value = binary_op!(self, f64, |a, b| a + b)?;
              self.push(Value::Number(value));
            }
            (Value::String(a), Value::String(b)) => {
              self.pop();
              self.pop();
              self.push(Value::String(format!("{}{}", a, b)));
            }
            _ => return Err(RuntimeError::OperationNotSupported.into()),
          }
        }
        OpCode::Subtract => {
          let value = binary_op!(self, f64, |a, b| a - b)?;
          self.push(Value::Number(value));
        }
        OpCode::Multiply => {
          let value = binary_op!(self, f64, |a, b| a * b)?;
          self.push(Value::Number(value));
        }
        OpCode::Divide => {
          let value = binary_op!(self, f64, |a, b| a / b)?;
          self.push(Value::Number(value));
        }
        OpCode::Exponent => {
          let value = binary_op!(self, f64, |a, b| f64::powf(a, b))?;
          self.push(Value::Number(value));
        }
        OpCode::Not => {
          let value = self.peek_and_pop_as::<bool>()?;
          self.push(Value::Boolean(!value));
        }
        OpCode::Negate => {
          let value = self.peek_and_pop_as::<f64>()?;
          self.push(Value::Number(-value));
        }
        OpCode::Return => {
          return Ok(self.pop());
        }
      }
    }
  }
}
