use std::num::ParseFloatError;
use std::convert;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    Lex(String),
    Parse(String),
    Float(ParseFloatError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        error::Error::description(self).fmt(f)
    }
}

impl error::Error for ParseError {
  fn description(&self) -> &str {
      match self {
          &ParseError::Lex(ref message) => message,
          &ParseError::Parse(ref message) => message,
          &ParseError::Float(ref err) => err.description(),
      }
  }
}

impl convert::From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        ParseError::Float(err)
    }
}

