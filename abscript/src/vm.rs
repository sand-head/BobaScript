use crate::{
  chunk::{Chunk, OpCode},
  debug::disassemble_instruction,
  value::Value,
};

macro_rules! binary_op {
  ($a: expr, $op: tt, $b: expr) => {{ $a $op $b }};
}

pub enum InterpretResult {
  Ok,
  CompileError,
  RuntimeError,
}

pub struct VM {
  chunk: Chunk,
  ip: usize,
  stack: Vec<Value>,
  debug: bool,
}
impl VM {
  pub fn new(debug: Option<bool>) -> Self {
    Self {
      chunk: Chunk::default(),
      ip: 0,
      stack: Vec::with_capacity(256),
      debug: debug.unwrap_or(false),
    }
  }

  pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
    self.chunk = chunk;
    self.ip = 0;
    self.run()
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value)
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }

  fn run(&mut self) -> InterpretResult {
    loop {
      let (instruction, line) = {
        let instruction = &self.chunk.code[self.ip];
        self.ip += 1;
        instruction
      };

      if self.debug {
        print!("\t");
        for value in self.stack.iter() {
          print!("[{}]", value);
        }
        println!();
        disassemble_instruction(&self.chunk, instruction, line, self.ip);
      }

      match instruction {
        OpCode::Constant(idx) => {
          self.push(self.chunk.constants[*idx].clone());
        }
        OpCode::Add => {
          let Value::Number(a) = self.pop();
          let Value::Number(b) = self.pop();

          self.push(Value::Number(binary_op!(a, +, b)));
        }
        OpCode::Subtract => {
          let Value::Number(a) = self.pop();
          let Value::Number(b) = self.pop();

          self.push(Value::Number(binary_op!(a, -, b)));
        }
        OpCode::Multiply => {
          let Value::Number(a) = self.pop();
          let Value::Number(b) = self.pop();

          self.push(Value::Number(binary_op!(a, *, b)));
        }
        OpCode::Divide => {
          let Value::Number(a) = self.pop();
          let Value::Number(b) = self.pop();

          self.push(Value::Number(binary_op!(a, /, b)));
        }
        OpCode::Negate => {
          if let Value::Number(num) = self.pop() {
            self.push(Value::Number(-num));
          }
        }
        OpCode::Return => {
          println!("{}", self.pop());
          return InterpretResult::Ok;
        }
        _ => continue,
      }
    }
  }
}
