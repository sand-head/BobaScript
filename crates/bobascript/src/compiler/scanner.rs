use bobascript_ast::tokens::{Token, TokenType};

use super::{CompileError, CompileResult};

pub struct Scanner {
  source: Vec<char>,
  start: usize,
  current: usize,
  line: usize,
}
impl Scanner {
  pub fn new(source: String) -> Self {
    Self {
      source: source.chars().collect(),
      start: 0,
      current: 0,
      line: 1,
    }
  }

  pub fn scan_token(&mut self) -> CompileResult<Token> {
    self.skip_whitespace();
    self.start = self.current;

    if self.is_at_end() {
      return Ok(self.make_token(TokenType::EOF));
    }

    let current_char = self.advance();
    match current_char {
      '0'..='9' => Ok(self.make_number()),
      'a'..='z' | 'A'..='Z' | '_' => Ok(self.make_identifier()),
      '"' => Ok(self.make_string()?),
      '(' => Ok(self.make_token(TokenType::LeftParen)),
      ')' => Ok(self.make_token(TokenType::RightParen)),
      '{' => Ok(self.make_token(TokenType::LeftBrace)),
      '}' => Ok(self.make_token(TokenType::RightBrace)),
      '[' => Ok(self.make_token(TokenType::LeftBracket)),
      ']' => Ok(self.make_token(TokenType::RightBracket)),
      '*' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::MultiplyAssign)
      } else {
        self.make_token(TokenType::Asterisk)
      }),
      '^' => Ok(self.make_token(TokenType::Carrot)),
      ',' => Ok(self.make_token(TokenType::Comma)),
      '$' => Ok(self.make_token(TokenType::DollarSign)),
      '-' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::SubtractAssign)
      } else {
        self.make_token(TokenType::Minus)
      }),
      '.' => Ok(self.make_token(TokenType::Period)),
      '+' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::AddAssign)
      } else {
        self.make_token(TokenType::Plus)
      }),
      ';' => Ok(self.make_token(TokenType::Semicolon)),
      '/' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::DivideAssign)
      } else {
        self.make_token(TokenType::Slash)
      }),
      '!' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::NotEqual)
      } else {
        self.make_token(TokenType::Not)
      }),
      '=' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::Equal)
      } else {
        self.make_token(TokenType::Assign)
      }),
      '<' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::LessEqual)
      } else {
        self.make_token(TokenType::LessThan)
      }),
      '>' => Ok(if self.current_matches('=') {
        self.make_token(TokenType::GreaterEqual)
      } else {
        self.make_token(TokenType::GreaterThan)
      }),
      _ => Err(CompileError::UnexpectedCharacter(self.line, current_char)),
    }
  }

  fn peek(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source[self.current]
    }
  }

  fn peek_next(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source[self.current + 1]
    }
  }

  fn skip_whitespace(&mut self) {
    loop {
      match self.peek() {
        ' ' | '\r' | '\t' => {
          self.advance();
        }
        '\n' => {
          self.line += 1;
          self.advance();
        }
        '/' if self.peek_next() == '/' => {
          while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
          }
        }
        _ => return,
      }
    }
  }

  fn is_at_end(&self) -> bool {
    self.current >= self.source.len() || self.source[self.current] == '\0'
  }

  fn advance(&mut self) -> char {
    let current_char = self.source[self.current];
    self.current += 1;
    current_char
  }

  fn current_matches(&mut self, expected: char) -> bool {
    if self.is_at_end() || self.source[self.current] != expected {
      false
    } else {
      self.current += 1;
      true
    }
  }

  fn check_keyword(
    &self,
    start: usize,
    length: usize,
    rest: &str,
    token_type: TokenType,
  ) -> TokenType {
    let substr: String = self.source[(self.start + start)..self.current]
      .iter()
      .collect();
    if self.current - self.start == start + length && substr == rest {
      token_type
    } else {
      TokenType::Identifier
    }
  }

  fn make_token(&self, token_type: TokenType) -> Token {
    Token {
      token_type,
      line: self.line,
      lexeme: self.source[self.start..self.current].iter().collect(),
    }
  }

  fn make_number(&mut self) -> Token {
    while let '0'..='9' = self.peek() {
      self.advance();
    }

    // also scan the fractional bit
    if self.peek() == '.' && self.peek_next() >= '0' && self.peek_next() <= '9' {
      self.advance();
      while let '0'..='9' = self.peek() {
        self.advance();
      }
    }

    self.make_token(TokenType::Number)
  }

  fn make_identifier(&mut self) -> Token {
    while let 'a'..='z' | 'A'..='Z' | '_' | '0'..='9' = self.peek() {
      self.advance();
    }

    // parse a keyword outta our identifier, or just leave it as-is
    self.make_token(match self.source[self.start] {
      'a' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'n' => self.check_keyword(2, 1, "d", TokenType::And),
        's' => self.check_keyword(2, 3, "ync", TokenType::Async),
        'w' => self.check_keyword(2, 3, "ait", TokenType::Await),
        _ => TokenType::Identifier,
      },
      'b' => self.check_keyword(1, 4, "reak", TokenType::Break),
      'c' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'l' => self.check_keyword(2, 3, "ass", TokenType::Class),
        'o' => self.check_keyword(2, 3, "nst", TokenType::Const),
        _ => TokenType::Identifier,
      },
      'e' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'l' => self.check_keyword(2, 2, "se", TokenType::Else),
        'n' => self.check_keyword(2, 2, "um", TokenType::Enum),
        _ => TokenType::Identifier,
      },
      'f' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'a' => self.check_keyword(2, 3, "lse", TokenType::False),
        'n' => self.check_keyword(2, 0, "", TokenType::Fn),
        'o' => self.check_keyword(2, 1, "r", TokenType::For),
        _ => TokenType::Identifier,
      },
      'i' => self.check_keyword(1, 1, "f", TokenType::If),
      'l' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'e' => self.check_keyword(2, 1, "t", TokenType::Let),
        'o' => self.check_keyword(2, 1, "g", TokenType::Log),
        _ => TokenType::Identifier,
      },
      'm' => self.check_keyword(1, 4, "atch", TokenType::Match),
      'o' => self.check_keyword(1, 1, "r", TokenType::Or),
      'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
      's' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'u' => self.check_keyword(2, 3, "per", TokenType::Super),
        'w' => self.check_keyword(2, 4, "itch", TokenType::Switch),
        _ => TokenType::Identifier,
      },
      't' if self.current - self.start > 1 => match self.source[self.start + 1] {
        'h' => self.check_keyword(2, 2, "is", TokenType::This),
        'r' => self.check_keyword(2, 2, "ue", TokenType::True),
        _ => TokenType::Identifier,
      },
      'w' => self.check_keyword(1, 4, "hile", TokenType::While),
      'y' => self.check_keyword(1, 4, "ield", TokenType::Yield),
      _ => TokenType::Identifier,
    })
  }

  fn make_string(&mut self) -> CompileResult<Token> {
    while self.peek() != '"' && !self.is_at_end() {
      if self.peek() == '\n' {
        self.line += 1;
      }
      self.advance();
    }

    if self.is_at_end() {
      Err(CompileError::UnterminatedString(self.line))
    } else {
      self.advance();
      Ok(self.make_token(TokenType::String))
    }
  }
}
