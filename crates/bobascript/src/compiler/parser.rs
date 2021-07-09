use bobascript_ast::tokens::{Token, TokenType};

use super::{scanner::Scanner, CompileError};

pub struct Parser {
  scanner: Scanner,
  current: Option<Token>,
  previous: Option<Token>,
  pub error: Option<CompileError>,
  panic_mode: bool,
}
impl Parser {
  pub fn new(source: String) -> Self {
    Self {
      scanner: Scanner::new(source),
      current: None,
      previous: None,
      error: None,
      panic_mode: false,
    }
  }

  pub fn current(&self) -> Option<&Token> {
    self.current.as_ref()
  }

  pub fn previous(&self) -> Option<&Token> {
    self.previous.as_ref()
  }

  pub fn current_type(&self) -> Option<TokenType> {
    self.current.as_ref().map(|c| c.token_type)
  }

  pub fn previous_type(&self) -> Option<TokenType> {
    self.previous.as_ref().map(|p| p.token_type)
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
          self.set_error(e);
        }
      }
    }
  }

  pub fn consume(&mut self, token_type: TokenType, err: CompileError) {
    if let Some(current_type) = self.current_type() {
      if current_type == token_type {
        return self.advance();
      }
    }

    self.set_error(err);
  }

  pub fn set_error(&mut self, err: CompileError) {
    if !self.panic_mode {
      self.panic_mode = true;
      self.error = Some(err);
    }
  }

  pub fn is_panicking(&self) -> bool {
    self.panic_mode
  }

  pub fn synchronize(&mut self) {
    self.panic_mode = false;

    loop {
      // if we're at the end of a statement, we're synched
      if let Some(TokenType::Semicolon) = self.previous_type() {
        return;
      }

      // also, if we're at the *start* of a statement, we're synched
      match self.current_type() {
        Some(
          TokenType::EOF
          | TokenType::Class
          | TokenType::Let
          | TokenType::Const
          | TokenType::For
          | TokenType::If
          | TokenType::While
          | TokenType::Return
          | TokenType::Break,
        ) => return,
        _ => (),
      }

      // not synched yet!
      self.advance();
    }
  }
}
