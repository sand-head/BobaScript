use crate::value::Value;

#[derive(Debug)]
pub enum OpCode {
  Unit,
  Constant(usize),
  True,
  False,
  Pop,
  DefineGlobal(usize),
  GetLocal(usize),
  SetLocal(usize),
  GetGlobal(usize),
  SetGlobal(usize),
  Equal,
  GreaterThan,
  LessThan,
  Add,
  Subtract,
  Multiply,
  Divide,
  Exponent,
  Not,
  Negate,
  Log,
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
