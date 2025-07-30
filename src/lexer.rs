use regex::Regex;
use std::error::Error as StdError; // for .description()
use std::iter;
use std::str::FromStr;

use crate::error::ParseError;

// IMPORTANT: This will not work performantly if it's called from any thread different than its
// first invocation (since Regex optimizes for the first thread).
lazy_static! {
    static ref REGEX_NUMBER: Regex = Regex::new(r"^[+-]?\d+(?:\.\d*)?(?:[eE]\d+)?").unwrap();
    static ref REGEX_IDENT: Regex = Regex::new(r"^[a-zA-Z]+").unwrap();
}

// These two must be in the same order.
const OPS_SINGLE: [char; 9] = ['+', '-', '*', '/', '^', '!', '=', '(', ')'];

/// Types of tokens.
#[derive(Debug, PartialEq)]
pub enum TokenType<'a> {
    Number(f64),
    Ident(&'a str),
    OpSingle(char),
    End,
}
use self::TokenType::*;

/// Token type with a text position number.
#[derive(Debug)]
pub struct Token<'a> {
    pub typ: TokenType<'a>,
    pub pos: u32,
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.typ == other.typ
    }
}

impl<'a> PartialEq<TokenType<'a>> for Token<'a> {
    fn eq(&self, other: &TokenType<'a>) -> bool {
        self.typ == *other
    }
}

impl<'a> PartialEq<Token<'a>> for TokenType<'a> {
    fn eq(&self, other: &Token<'a>) -> bool {
        *self == other.typ
    }
}

/// Lexer class that exposes an iterator for easy token consumption.
#[derive(Clone)]
pub struct Lexer<'a> {
    text: &'a str,
    pos: u32,
    error: Option<String>,
}

impl<'a> Lexer<'a> {
    /// Creates a lexer and also parses the first token. Returns an error if the parse fails.
    pub fn new<'b>(text: &'b str) -> Lexer<'b> {
        Lexer {
            text: text,
            pos: 0,
            error: None,
        }
    }

    /// Returns the current token, or an error if there was a lexing error.
    /// Subsequent invocations after an error, return a generic error.
    pub fn next_token(&mut self) -> Result<Token<'a>, ParseError> {
        if let Some(ref msg) = self.error {
            Err(ParseError::Lex(msg.clone()))
        } else {
            let res = self.next_token_();
            if let Err(ref err) = res {
                let mut msg = "Errored previously: ".to_owned();
                msg.push_str(err.description());
                self.error = Some(msg);
            }
            res
        }
    }

    fn next_token_(&mut self) -> Result<Token<'a>, ParseError> {
        let mut ch;
        {
            let mut iter = self.text.chars();
            loop {
                ch = match iter.next() {
                    // End of stream
                    None => {
                        return Ok(Token {
                            typ: End,
                            pos: self.pos,
                        })
                    }
                    Some(ch) => ch,
                };
                // Skip whitespace and increment pos
                if !ch.is_whitespace() {
                    break;
                }
                self.text = &self.text[1..];
                self.pos += 1;
            }
        }

        // Check single-character tokens.
        // NOTE: Relocate this to the end if any of these become prefixes of longer tokens.
        if OPS_SINGLE.contains(&ch) {
            let token = Token {
                typ: OpSingle(ch),
                pos: self.pos,
            };
            self.text = &self.text[1..];
            self.pos += 1;
            Ok(token)
        } else if let Some((0, n)) = REGEX_NUMBER.find(self.text) {
            let token = Token {
                typ: Number(FromStr::from_str(&self.text[..n])?),
                pos: self.pos,
            };
            self.text = &self.text[n..];
            self.pos += n as u32;
            Ok(token)
        } else if let Some((0, n)) = REGEX_IDENT.find(self.text) {
            let token = Token {
                typ: Ident(&self.text[..n]),
                pos: self.pos,
            };
            self.text = &self.text[n..];
            self.pos += n as u32;
            Ok(token)
        } else {
            Err(ParseError::Lex(format!(
                "Unexpected '{}' at position {}",
                ch, self.pos
            )))
        }
    }

    /// Iterator returning sequential values of `next_token()`.
    pub fn iter(&'a mut self) -> Iter<'a> {
        Iter(self)
    }
}

impl<'a> iter::Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, ParseError>;

    fn next(&mut self) -> Option<Result<Token<'a>, ParseError>> {
        // The iterations stop when either the end has been reached or an error is encountered.
        if self.error.is_some() {
            return None;
        }
        match self.next_token() {
            Ok(Token { typ: End, pos: _ }) => None,
            res => Some(res),
        }
    }
}

/// Iterator for Lexer (to avoid losing ownership of the Lexer, e.g., for `Iterator::collect()`.
pub struct Iter<'a>(&'a mut Lexer<'a>);

/// This is mostly for debugging -- hence the println on failure to avoid losing information about
/// the result.
impl<'a> iter::Iterator for Iter<'a> {
    type Item = Result<Token<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_print_tokens() {
        use super::TokenType::Ident;
        use super::{Lexer, Token};

        let text = "log(3x!!+4)- 5x zy^2^3";
        let lexer = Lexer::new(text);

        let mut error = None;
        let vec: Vec<_> = lexer
            .map(|res| {
                res.unwrap_or_else(|err| {
                    error = Some(err);
                    Token {
                        typ: Ident("<ERROR>"),
                        pos: 0,
                    }
                })
            })
            .collect();
        println!("---------------------");
        println!("{}", text);
        println!("----");
        for token in &vec {
            println!("{:?}", token);
        }
        if let Some(err) = error {
            println!("ERROR: {:?}", err);
        }
        println!("---------------------");
    }

    #[test]
    fn test_lexer_iter_eq() {
        use super::TokenType::Ident;
        use super::{Lexer, Token};

        let text = "log(3x!!+4)- 5x zy^2^3";
        let lexer = Lexer::new(text);
        let mut lexer_clone = lexer.clone();
        let iter = lexer_clone.iter();

        let vec1: Vec<_> = lexer
            .map(|res| {
                res.unwrap_or_else(|_| Token {
                    typ: Ident("<ERROR>"),
                    pos: 0,
                })
            })
            .collect();
        let vec2: Vec<_> = iter
            .map(|res| {
                res.unwrap_or_else(|_| Token {
                    typ: Ident("<ERROR>"),
                    pos: 0,
                })
            })
            .collect();
        assert_eq!(vec1, vec2);
    }
}
