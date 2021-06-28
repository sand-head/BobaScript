use abscript::{
  chunk::{Chunk, OpCode},
  value::Value,
  vm::VM,
};

fn main() {
  let mut vm = VM::new(Option::Some(true));
  let mut chunk = Chunk::default();

  let constant = chunk.add_constant(Value::Number(1.2));
  chunk.write(OpCode::Constant(constant), 123);
  chunk.write(OpCode::Negate, 123);
  let constant = chunk.add_constant(Value::Number(3.4));
  chunk.write(OpCode::Constant(constant), 123);
  chunk.write(OpCode::Add, 123);
  chunk.write(OpCode::Return, 123);

  vm.interpret(chunk);
}
