use num_enum::{FromPrimitive, IntoPrimitive};

use super::Compiler;

#[macro_export]
macro_rules! parse_prefix {
  ($prefix:expr,$precedence:ident) => {
    ParseRule {
      prefix: Some($prefix),
      infix: None,
      precedence: Precedence::$precedence,
    }
  };
}

#[macro_export]
macro_rules! parse_infix {
  ($infix:expr,$precedence:ident) => {
    ParseRule {
      prefix: None,
      infix: Some($infix),
      precedence: Precedence::$precedence,
    }
  };
}

#[macro_export]
macro_rules! parse_both {
  ($prefix:expr,$infix:expr,$precedence:ident) => {
    ParseRule {
      prefix: Some($prefix),
      infix: Some($infix),
      precedence: Precedence::$precedence,
    }
  };
}

#[macro_export]
macro_rules! parse_none {
  () => {
    ParseRule {
      prefix: None,
      infix: None,
      precedence: Precedence::None,
    }
  };
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, IntoPrimitive, FromPrimitive)]
#[repr(usize)]
pub enum Precedence {
  #[num_enum(default)]
  None,
  Assignment,
  Or,
  And,
  Equality,
  Comparison,
  Term,
  Factor,
  Unary,
  Call,
  Primary,
}

type ParseFn = fn(&mut Compiler) -> ();
pub struct ParseRule {
  pub prefix: Option<ParseFn>,
  pub infix: Option<ParseFn>,
  pub precedence: Precedence,
}
