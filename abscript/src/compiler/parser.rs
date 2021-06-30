use super::{
  scanner::{Scanner, Token, TokenType},
  CompileError,
};

pub struct Parser {
  scanner: Scanner,
  pub current: Option<Token>,
  pub previous: Option<Token>,
  pub had_error: bool,
  panic_mode: bool,
}
impl Parser {
  pub fn new(source: String) -> Self {
    Self {
      scanner: Scanner::new(source),
      current: None,
      previous: None,
      had_error: false,
      panic_mode: false,
    }
  }

  pub fn advance(&mut self) {
    if let Some(token) = &self.current {
      self.previous = Some(token.clone());
    }

    loop {
      match self.scanner.scan_token() {
        Ok(token) => {
          self.current = Some(token);
          break;
        }
        Err(e) => {
          self.current = None;
          self.print_error(e);
        }
      }
    }
  }

  pub fn consume(&mut self, token_type: TokenType, err: CompileError) {
    if let Some(token) = &self.current {
      if token.token_type == token_type {
        return self.advance();
      }
    }

    self.print_error(err);
  }

  pub fn print_error(&mut self, err: CompileError) {
    if !self.panic_mode {
      self.panic_mode = true;
      self.had_error = true;
      eprintln!("{}", err);
    }
  }
}
