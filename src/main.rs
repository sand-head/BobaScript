use std::io::{self, Write};

use abscript::{value::Value, vm::VM, InterpretError, InterpretResult};
use rustyline::{
  error::ReadlineError,
  validate::{MatchingBracketValidator, ValidationContext, ValidationResult, Validator},
  Editor, Result,
};
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {
  brackets: MatchingBracketValidator,
}

impl Validator for InputValidator {
  fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
    self.brackets.validate(ctx)
  }
}

fn main() -> InterpretResult<()> {
  // set up editor
  let helper = InputValidator {
    brackets: MatchingBracketValidator::new(),
  };
  let mut rl = Editor::new();
  rl.set_helper(Some(helper));

  // create our virtual machine
  let mut vm = VM::new();

  println!("Howdy! Welcome to the ABScript REPL, enjoy your stay.");
  loop {
    match rl.readline("> ") {
      Ok(input) => match vm.interpret(input) {
        Ok(Value::Unit) => (),
        Ok(value) => println!("< {}", value),
        Err(err) => print_error(format!("{}", err)).map_err(|_| InterpretError::Unknown)?,
      },
      Err(ReadlineError::Interrupted) => {
        println!("Stopping REPL...");
        break Ok(());
      }
      Err(_) => {
        print_error("Could not read input from console.".to_string())
          .map_err(|_| InterpretError::Unknown)?;
        break Ok(());
      }
    }
  }
}

fn print_error(msg: String) -> io::Result<()> {
  let mut stderr = StandardStream::stderr(ColorChoice::Auto);
  stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
  writeln!(&mut stderr, "[!] {}", msg)?;
  stderr.reset()
}
