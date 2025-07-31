use std::num::ParseFloatError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("{0}")]
    Lex(String),
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    Float(#[from] ParseFloatError),
}

