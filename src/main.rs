use std::{
  convert::TryInto,
  io::{self, Write},
};

use bobascript::{
  compiler::{compile, compile_expr},
  value::Value,
  vm::VM,
  InterpretError, InterpretResult,
};
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

fn log_handler(value: Value) {
  let output: String = value.try_into().unwrap();
  println!("# {}", output);
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
  vm.add_log_handler(Box::new(log_handler));

  println!("Howdy! Welcome to the BobaScript REPL, enjoy your stay.");
  loop {
    match rl.readline("> ") {
      Ok(input) => {
        rl.add_history_entry(&input);
        if input.ends_with(";") {
          let function = compile(input)?;
          match vm.interpret(function) {
            Err(err) => print_error(format!("{}", err)).map_err(|_| InterpretError::Unknown)?,
            _ => (),
          }
        } else {
          let function = compile_expr(input)?;
          match vm.evaluate(function) {
            Ok(value) => println!("< {}", value),
            Err(err) => print_error(format!("{}", err)).map_err(|_| InterpretError::Unknown)?,
          }
        }
      }
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
