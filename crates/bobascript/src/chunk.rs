use crate::value::Value;

#[derive(Debug, Clone)]
pub enum OpCode {
  Tuple(u8),
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
  GetUpvalue(usize),
  SetUpvalue(usize),
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
  Jump(JumpDirection, usize),
  JumpIfFalse(usize),
  Call(u8),
  Closure(usize, Vec<Upvalue>),
  CloseUpvalue,
  Return,
}

#[derive(Debug, Clone, Copy)]
pub enum JumpDirection {
  Forwards,
  Backwards,
}

#[derive(Debug, Clone, Copy)]
pub enum Upvalue {
  Local(usize),
  Upvalue(usize),
}

#[derive(Debug)]
pub struct Chunk {
  // todo: re-add line numbers to code somehow
  pub code: Vec<OpCode>,
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
  pub fn write(&mut self, opcode: OpCode) -> usize {
    self.code.push(opcode);
    self.code.len() - 1
  }

  pub fn add_constant(&mut self, value: Value) -> usize {
    self.constants.push(value);
    self.constants.len() - 1
  }
}
