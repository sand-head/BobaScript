use crate::value::Value;

pub enum OpCode {
  Constant(usize),
  Return,
}

pub struct Chunk {
  pub code: Vec<(OpCode, usize)>,
  pub constants: Vec<Value>,
}
impl Default for Chunk {
  fn default() -> Self {
    Self {
      code: Vec::new(),
      constants: Vec::new(),
    }
  }
}

impl Chunk {
  pub fn write(&mut self, opcode: OpCode, line: usize) {
    self.code.push((opcode, line))
  }

  pub fn add_constant(&mut self, value: Value) -> usize {
    self.constants.push(value);
    self.constants.len() - 1
  }
}
