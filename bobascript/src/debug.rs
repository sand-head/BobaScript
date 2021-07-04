use crate::chunk::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
  println!("== {} ==", name);

  for (i, (opcode, line)) in chunk.code.iter().enumerate() {
    disassemble_instruction(chunk, opcode, line, i);
  }
}

pub fn disassemble_instruction(chunk: &Chunk, opcode: &OpCode, line: &usize, offset: usize) {
  let instruction = match opcode {
    OpCode::Constant(idx) => format!("Constant {:0>#4} {}", idx, chunk.constants[*idx]),
    _ => format!("{:?}", opcode),
  };
  println!("{:0>#4} #{:0>#4} {}", offset, line, instruction);
}
