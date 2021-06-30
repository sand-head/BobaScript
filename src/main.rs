use std::io::{stdin, stdout, Write};

use abscript::{vm::VM, InterpretResult};

fn main() -> InterpretResult<()> {
  let mut vm = VM::new();

  loop {
    let mut input = String::new();
    print!("> ");
    stdout().flush().unwrap();

    stdin()
      .read_line(&mut input)
      .expect("Could not read input from console");

    match vm.interpret(input) {
      Err(e) => eprintln!("Error: {}", e),
      _ => continue,
    }
  }
}
