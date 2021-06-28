use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
  println!("== {} ==", name);

  for (i, (opcode, line)) in chunk.code.iter().enumerate() {
    disassemble_instruction(chunk, opcode, line, i);
  }
}

pub fn disassemble_instruction(chunk: &Chunk, opcode: &OpCode, line: &usize, offset: usize) {
  let instruction = match opcode {
    OpCode::Constant(idx) => format!(
      "{} {:0>#4} {}",
      "OpCode::Constant", idx, chunk.constants[*idx]
    ),
    OpCode::Add => "OpCode::Add".to_string(),
    OpCode::Subtract => "OpCode::Subtract".to_string(),
    OpCode::Multiply => "OpCode::Multiply".to_string(),
    OpCode::Divide => "OpCode::Divide".to_string(),
    OpCode::Negate => "OpCode::Negate".to_string(),
    OpCode::Return => "OpCode::Return".to_string(),
  };
  println!("{:0>#4} #{:0>#4} {}", offset, line, instruction);
}
