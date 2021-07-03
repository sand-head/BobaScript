use crate::value::Value;

#[derive(Debug)]
pub enum OpCode {
  Unit,
  Constant(usize),
  True,
  False,
  Pop,
  PopN(usize),
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
  Jump(usize),
  JumpIfFalse(usize),
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
  pub fn write(&mut self, opcode: OpCode, line: usize) -> usize {
    self.code.push((opcode, line));
    self.code.len() - 1
  }

  pub fn add_constant(&mut self, value: Value) -> usize {
    self.constants.push(value);
    self.constants.len() - 1
  }
}
