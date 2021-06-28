use abscript::{
  chunk::{Chunk, OpCode},
  debug::disassemble_chunk,
  value::Value,
};

fn main() {
  let mut chunk = Chunk::default();

  let constant = chunk.add_constant(Value::Number(1.2));
  chunk.write(OpCode::Constant(constant), 123);
  chunk.write(OpCode::Return, 123);

  disassemble_chunk(&chunk, "test chunk");
}
