use crate::{
  chunk::{Chunk, OpCode},
  compiler::Compiler,
  debug::disassemble_instruction,
  value::Value,
  InterpretResult,
};

macro_rules! binary_op {
  ($a: expr, $op: tt, $b: expr) => {{ $a $op $b }};
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

  pub fn interpret<S>(&mut self, source: S) -> InterpretResult<()>
  where
    S: Into<String>,
  {
    let mut chunk = Chunk::default();
    let mut compiler = Compiler::new(&mut chunk);
    compiler.compile(source.into());

    self.chunk = chunk;
    self.ip = 0;
    Ok(self.run()?)
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value)
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }

  fn run(&mut self) -> InterpretResult<()> {
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
          self.push(self.chunk.constants[*idx].clone());
        }
        OpCode::Add => {
          let Value::Number(b) = self.pop();
          let Value::Number(a) = self.pop();

          self.push(Value::Number(binary_op!(a, +, b)));
        }
        OpCode::Subtract => {
          let Value::Number(b) = self.pop();
          let Value::Number(a) = self.pop();

          self.push(Value::Number(binary_op!(a, -, b)));
        }
        OpCode::Multiply => {
          let Value::Number(b) = self.pop();
          let Value::Number(a) = self.pop();

          self.push(Value::Number(binary_op!(a, *, b)));
        }
        OpCode::Divide => {
          let Value::Number(b) = self.pop();
          let Value::Number(a) = self.pop();

          self.push(Value::Number(binary_op!(a, /, b)));
        }
        OpCode::Exponent => {
          let Value::Number(b) = self.pop();
          let Value::Number(a) = self.pop();

          self.push(Value::Number(f64::powf(a, b)));
        }
        OpCode::Negate => {
          if let Value::Number(num) = self.pop() {
            self.push(Value::Number(-num));
          }
        }
        OpCode::Return => {
          println!("{}", self.pop());
          return Ok(());
        }
        _ => continue,
      }
    }
  }
}
