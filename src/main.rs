use std::io::{stdin, stdout, Write};

use abscript::vm::VM;

fn main() {
  let mut vm = VM::new(Option::Some(true));

  loop {
    let mut input = String::new();
    print!("> ");
    stdout().flush().unwrap();

    stdin()
      .read_line(&mut input)
      .expect("Could not read input from console");

    vm.interpret(input);
  }
}
