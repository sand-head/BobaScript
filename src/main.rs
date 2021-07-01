use std::io::{self, stdin, stdout, Write};

use abscript::{vm::VM, InterpretError, InterpretResult};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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
      Ok(value) => println!("< {}", value),
      Err(err) => print_error(err).map_err(|_| InterpretError::Unknown)?,
    }
  }
}

fn print_error(err: InterpretError) -> io::Result<()> {
  let mut stderr = StandardStream::stderr(ColorChoice::Auto);
  stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
  writeln!(&mut stderr, "[!] {}", err)?;
  stderr.reset()
}
