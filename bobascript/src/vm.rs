use std::{cell::RefCell, collections::HashMap, convert::TryInto, iter::repeat, rc::Rc};

use thiserror::Error;

use crate::{
  chunk::{JumpDirection, OpCode},
  compiler::Compiler,
  debug::disassemble_instruction,
  value::{Function, NativeFunction, Value},
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
  #[error("Only functions and classes may be called.")]
  InvalidCallSignature,
  #[error("Expected {0} arguments, but got {1}.")]
  IncorrectParameterCount(u8, u8),
  #[error("Stack overflow.")]
  StackOverflow,
}

struct CallFrame {
  function: Rc<Function>,
  ip: usize,
  slots_start: usize,
}

pub struct VM<'a> {
  log_handler: Option<&'a dyn Fn(Value) -> ()>,
  frames: Vec<CallFrame>,
  stack: Vec<Value>,
  globals: HashMap<String, Value>,
}
impl<'a> VM<'a> {
  pub fn new() -> Self {
    Self {
      log_handler: None,
      frames: Vec::with_capacity(64),
      stack: Vec::with_capacity(256),
      globals: HashMap::new(),
    }
  }

  pub fn add_log_handler(&mut self, handler: &'a dyn Fn(Value) -> ()) {
    self.log_handler = Some(handler);
  }

  pub fn define_native(&mut self, name: String, function: Rc<RefCell<NativeFunction>>) {
    self.push(Value::String(name));
    self.push(Value::NativeFunction(function));
    self.globals.insert(
      self.stack[0].clone().try_into().unwrap(),
      self.stack[1].clone().try_into().unwrap(),
    );
    self.pop_n(2);
  }

  pub fn interpret<S>(&mut self, source: S) -> InterpretResult<Value>
  where
    S: Into<String>,
  {
    let mut compiler = Compiler::new();
    let function = compiler.compile(source.into())?;

    self.push(Value::Function(function.clone()));
    self.call(function, 0)?;

    let result = self.run();
    self.stack.clear();
    self.frames.clear();
    result
  }

  fn frame(&self) -> &CallFrame {
    &self.frames[self.frames.len() - 1]
  }

  fn frame_mut(&mut self) -> &mut CallFrame {
    let current_frame = self.frames.len() - 1;
    &mut self.frames[current_frame]
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

  fn pop_n(&mut self, count: usize) {
    for _ in 0..count {
      self.pop();
    }
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

  fn call_value(&mut self, callee: Value, arg_count: u8) -> InterpretResult<()> {
    match callee {
      Value::Function(function) => {
        self.call(function, arg_count)?;
        Ok(())
      }
      Value::NativeFunction(native_fn) => {
        let arg_start = self.stack.len() - 1 - (arg_count as usize);
        let args = &self.stack[arg_start..];
        (native_fn.borrow().function)(args)?;
        self.pop_n(arg_count as usize);
        Ok(())
      }
      _ => Err(RuntimeError::InvalidCallSignature.into()),
    }
  }

  fn call(&mut self, function: Rc<Function>, arg_count: u8) -> InterpretResult<()> {
    if arg_count != function.arity {
      return Err(RuntimeError::IncorrectParameterCount(function.arity, arg_count).into());
    }
    if self.frames.len() == 64 {
      return Err(RuntimeError::StackOverflow.into());
    }

    self.frames.push(CallFrame {
      function,
      ip: 0,
      slots_start: self.stack.len() - 1 - (arg_count as usize),
    });
    Ok(())
  }

  fn run(&mut self) -> InterpretResult<Value> {
    loop {
      let (instruction, line) = {
        let frame = self.frame();
        let instruction = frame.function.chunk.code[frame.ip].clone();
        self.frame_mut().ip += 1;
        instruction
      };

      if crate::DEBUG {
        print!("\t");
        for value in self.stack.iter() {
          print!("[{}]", value);
        }
        println!();
        disassemble_instruction(
          &self.frame().function.chunk,
          &instruction,
          &line,
          self.frame().ip,
        );
      }

      match instruction {
        OpCode::Unit => self.push(Value::Unit),
        OpCode::Constant(idx) => {
          let constant = self.frame().function.chunk.constants[idx].clone();
          self.push(constant);
        }
        OpCode::True => self.push(Value::Boolean(true)),
        OpCode::False => self.push(Value::Boolean(false)),
        OpCode::Pop => {
          self.pop();
        }
        OpCode::PopN(count) => {
          self.pop_n(count);
        }
        OpCode::DefineGlobal(global) => {
          let global = self.frame().function.chunk.constants[global].clone();
          let name: String = global.try_into()?;
          self.globals.insert(name, self.peek(0).unwrap().clone());
          self.pop();
        }
        OpCode::GetLocal(local) => {
          let local = self.stack[self.frame().slots_start + local].clone();
          self.push(local);
        }
        OpCode::SetLocal(local) => {
          let slot_offset = self.frame().slots_start;
          self.stack[slot_offset + local] = self.peek(0).unwrap().clone();
        }
        OpCode::GetGlobal(global) => {
          let global = self.frame().function.chunk.constants[global].clone();
          let name: String = global.try_into()?;

          let value = self
            .globals
            .get(&name)
            .ok_or_else(|| RuntimeError::UndefinedVariable(name))?
            .clone();

          self.push(value);
        }
        OpCode::SetGlobal(global) => {
          let global = self.frame().function.chunk.constants[global].clone();
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
            _ => break Err(RuntimeError::OperationNotSupported.into()),
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
            _ => break Err(RuntimeError::OperationNotSupported.into()),
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
        OpCode::Jump(direction, offset) => match direction {
          JumpDirection::Forwards => self.frame_mut().ip += offset,
          JumpDirection::Backwards => self.frame_mut().ip -= offset,
        },
        OpCode::JumpIfFalse(offset) => {
          let condition: bool = self.peek(0).unwrap().clone().try_into().unwrap();
          if !condition {
            self.frame_mut().ip += offset;
          }
        }
        OpCode::Call(args) => {
          self.call_value(self.peek(args as usize).unwrap().clone(), args)?;
        }
        OpCode::Return => {
          let result = self.pop().unwrap_or_else(|| Value::Unit);
          if self.frames.len() == 1 {
            // if this is the last frame, pop it and break the loop
            self.frames.pop();
            self.pop();
            break Ok(result);
          }

          // otherwise, pop everything in that frame's stack window and push the result back
          let pop_count = self.stack.len() - self.frame().slots_start;
          self.pop_n(pop_count);
          self.push(result);
          self.frames.pop();
        }
      }
    }
  }
}
