use crate::scanner::{ScanError, Scanner, TokenType};

pub fn compile(source: String) -> Result<(), ScanError> {
  let mut scanner = Scanner::new(source);
  let mut line = 0;

  loop {
    let token = scanner.scan_token()?;
    if token.line != line {
      print!("{:0>#4} ", token.line);
      line = token.line;
    } else {
      print!("   | ");
    }
    println!("{:?} {}", token.token_type, token.lexeme);

    if let TokenType::EOF = token.token_type {
      break;
    }
  }

  Ok(())
}
