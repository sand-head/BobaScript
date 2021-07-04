use std::{collections::HashMap, convert::TryInto, iter::repeat};

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
  #[error("Undefined variable \"{0}\".")]
  UndefinedVariable(String),
}

pub struct VM<'a> {
  log_handler: Option<&'a dyn Fn(Value) -> ()>,
  chunk: Chunk,
  ip: usize,
  stack: Vec<Value>,
  globals: HashMap<String, Value>,
}
impl<'a> VM<'a> {
  pub fn new() -> Self {
    Self {
      log_handler: None,
      chunk: Chunk::default(),
      ip: 0,
      stack: Vec::with_capacity(256),
      globals: HashMap::new(),
    }
  }

  pub fn add_log_handler(&mut self, handler: &'a dyn Fn(Value) -> ()) {
    self.log_handler = Some(handler);
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

  fn pop(&mut self) -> Option<Value> {
    self.stack.pop()
  }

  fn pop_as<T>(&mut self) -> InterpretResult<T>
  where
    Value: TryInto<T, Error = RuntimeError>,
  {
    Ok(
      self
        .pop()
        .ok_or_else(|| RuntimeError::Unknown)?
        .try_into()?,
    )
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
        OpCode::Unit => self.push(Value::Unit),
        OpCode::Constant(idx) => {
          let constant = self.chunk.constants[*idx].clone();
          self.push(constant);
        }
        OpCode::True => self.push(Value::Boolean(true)),
        OpCode::False => self.push(Value::Boolean(false)),
        OpCode::Pop => {
          self.pop();
        }
        OpCode::PopN(count) => {
          for _ in 0..*count {
            self.pop();
          }
        }
        OpCode::DefineGlobal(global) => {
          let global = self.chunk.constants[*global].clone();
          let name: String = global.try_into()?;
          self.globals.insert(name, self.peek(0).unwrap().clone());
          self.pop();
        }
        OpCode::GetLocal(local) => {
          let local = self.stack[*local].clone();
          self.push(local);
        }
        OpCode::SetLocal(local) => {
          self.stack[*local] = self.peek(0).unwrap().clone();
        }
        OpCode::GetGlobal(global) => {
          let global = self.chunk.constants[*global].clone();
          let name: String = global.try_into()?;

          let value = self
            .globals
            .get(&name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(name))?
            .clone();

          self.push(value);
        }
        OpCode::SetGlobal(global) => {
          let global = self.chunk.constants[*global].clone();
          let name: String = global.try_into()?;
          let new_value = self.peek(0).unwrap().clone();

          *self
            .globals
            .get_mut(&name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(name))? = new_value;
        }
        OpCode::Equal => {
          let b = self.pop().ok_or_else(|| RuntimeError::Unknown)?;
          let a = self.pop().ok_or_else(|| RuntimeError::Unknown)?;
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
          let b = self.peek(0).ok_or_else(|| RuntimeError::Unknown)?;
          let a = self.peek(1).ok_or_else(|| RuntimeError::Unknown)?;

          match (a, b) {
            (Value::Number(_), Value::Number(_)) => {
              let value = binary_op!(self, f64, |a, b| a + b)?;
              self.push(Value::Number(value));
            }
            (Value::String(_), Value::String(_))
            | (Value::Number(_), Value::String(_))
            | (Value::String(_), Value::Number(_)) => {
              let b = self.pop_as::<String>()?;
              let a = self.pop_as::<String>()?;
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
          let b = self.peek(0).ok_or_else(|| RuntimeError::Unknown)?;
          let a = self.peek(1).ok_or_else(|| RuntimeError::Unknown)?;

          match (a, b) {
            (Value::Number(_), Value::Number(_)) => {
              let value = binary_op!(self, f64, |a, b| a * b)?;
              self.push(Value::Number(value));
            }
            (Value::String(_), Value::Number(_)) => {
              let b = self.pop_as::<f64>()?;
              let a = self.pop_as::<String>()?;
              let value: String = repeat(a).take(b.round() as usize).collect();
              self.push(Value::String(value));
            }
            _ => return Err(RuntimeError::OperationNotSupported.into()),
          }
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
        OpCode::Log => {
          let value = self.peek(0).unwrap().clone();
          if let Some(handler) = self.log_handler {
            handler(value);
          } else {
            println!("{}", value);
          }
        }
        OpCode::Jump(offset) => {
          self.ip += offset;
        }
        OpCode::JumpIfFalse(offset) => {
          let condition: bool = self.peek(0).unwrap().clone().try_into().unwrap();
          if !condition {
            self.ip += offset;
          }
        }
        OpCode::Loop(offset) => {
          self.ip -= offset;
        }
        OpCode::Return => {
          return Ok(Value::Unit);
        }
      }
    }
  }
}
