use ast::AstNode;
use ast::AstType::{Number, Ident, Func, Binary, Prefix, Postfix, Parens};
use error::ParseError;
use lexer::{Lexer, Token, TokenType};

////////////////////////////////////////////////////////////////////////////////
// Operators

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum OpType {
    Sentinel,
    Binary,
    Prefix,
    Postfix,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Assoc { Left, Right }

#[derive(Debug)]
struct Op {
    ch: char,
    typ: OpType,
    prec: u32,
    assoc: Assoc,
}

static SENTINEL: Op =
    Op { ch: '\0', typ: OpType::Sentinel, prec: 0, assoc: Assoc::Left };

static OPS: [Op; 7] = [
    Op { ch: '+', typ: OpType::Binary,  prec: 1, assoc: Assoc::Left  },
    Op { ch: '-', typ: OpType::Binary,  prec: 1, assoc: Assoc::Left  },
    Op { ch: '*', typ: OpType::Binary,  prec: 2, assoc: Assoc::Left  },
    Op { ch: '/', typ: OpType::Binary,  prec: 2, assoc: Assoc::Left  },
    Op { ch: '-', typ: OpType::Prefix,  prec: 3, assoc: Assoc::Left  },
    Op { ch: '!', typ: OpType::Postfix, prec: 4, assoc: Assoc::Left  },
    Op { ch: '^', typ: OpType::Binary,  prec: 5, assoc: Assoc::Right },
];

fn is_sentinel(op: Option<&(&Op, u32)>) -> bool {
    if let Some(&(&Op { typ: OpType::Sentinel, .. }, _)) = op {
        true
    } else {
        false
    }
}

fn get_op(op_char: char, typ: OpType) -> Option<&'static Op> {
    OPS.iter().find(move |op| op.ch == op_char && op.typ == typ)
}

#[inline(always)]
fn has_greater_prec(op1: &Op, op2: &Op) -> bool {
    op1.prec > op2.prec || (op1.prec == op2.prec && op1.assoc == Assoc::Left)
}

////////////////////////////////////////////////////////////////////////////////
// Shunting Yard parser

struct ShuntingYard<'a> {
    lexer: Lexer<'a>,
    next: Token<'a>,
    op_stack: Vec<(&'static Op, u32)>, // (operator, position) pair
    exp_stack: Vec<AstNode>,
}

impl<'a> ShuntingYard<'a> {
    fn parse(&mut self) -> Result<AstNode, ParseError> {
        try!(self.parse_e());
        try!(self.expect(TokenType::End));
        assert_eq!(self.exp_stack.len(), 1);
        assert_eq!(self.op_stack.len(), 1);
        Ok::<AstNode, ParseError>(self.exp_stack.pop().unwrap())
    }

    fn consume(&mut self) -> Result<(), ParseError> {
        self.next = try!(self.lexer.next_token());
        Ok(())
    }

    fn expect(&mut self, token_type: TokenType<'a>) -> Result<(), ParseError> {
        if self.next == token_type {
            try!(self.consume());
            Ok(())
        } else {
            Err(ParseError::Parse(format!("Expected {:?} of expression, but got {:?} at position {:?}",
                                          token_type, self.next.typ, self.next.pos)))
        }
    }

    fn parse_e(&mut self) -> Result<(), ParseError> {
        try!(self.parse_p());
        while let Token { typ: TokenType::OpSingle(ch), pos } = self.next {
            if let Some(op) = get_op(ch, OpType::Binary) {
                self.push_operator((op, pos));
                try!(self.consume());
                try!(self.parse_p());
            } else if let Some(op) = get_op(ch, OpType::Postfix) {
                self.push_operator((op, pos));
                // The postfix operator's sole argument should be ready on the expression stack
                // after push_operator completes, taking precedence into account.
                self.pop_operator();
                try!(self.consume());
            } else {
                break;
            }
        }
        while !is_sentinel(self.op_stack.last()) {
            self.pop_operator()
        }
        Ok(())
    }

    fn parse_p(&mut self) -> Result<(), ParseError> {
        match &self.next {
            &Token { typ: TokenType::Number(v), pos } => {
                self.exp_stack.push(AstNode::new(Number(v), pos));
                try!(self.consume());
            },
            &Token { typ: TokenType::Ident(s), pos } => {
                try!(self.consume());
                if self.match_starting_parens() {
                    // Function call
                    let t = try!(self.parse_parens(pos));
                    self.exp_stack.push(AstNode::new(Func(s.to_string(), t), pos));
                } else {
                    // Identifier
                    self.exp_stack.push(AstNode::new(Ident(s.to_string()), pos));
                }
            },
            &Token { typ: TokenType::OpSingle('('), pos } => {
                let t = try!(self.parse_parens(pos));
                self.exp_stack.push(AstNode::new(Parens(t), pos));
            },
            &Token { typ: TokenType::OpSingle(ch), pos } => {
                if let Some(op) = get_op(ch, OpType::Prefix) {
                    self.push_operator((op, pos));
                    try!(self.consume());
                    try!(self.parse_p());
                } else {
                    return Err(ParseError::Parse(format!("Expected unary operator, but got {:?}", ch)));
                }
            },
            _ => {
                return Err(ParseError::Parse(format!("Unexpected token {:?}", self.next)));
            }
        }
        Ok(())
    }

    fn match_starting_parens(&mut self) -> bool {
        if let &Token { typ: TokenType::OpSingle('('), pos: _ } = &self.next { true } else { false }
    }

    fn parse_parens(&mut self, pos: u32) -> Result<Box<AstNode>, ParseError> {
        assert!(self.match_starting_parens());
        try!(self.consume());
        self.op_stack.push((&SENTINEL, pos));
        try!(self.parse_e());
        try!(self.expect(TokenType::OpSingle(')')));
        self.op_stack.pop().unwrap();
        Ok(Box::new(self.exp_stack.pop().unwrap()))
    }

    fn top_operator(&mut self) -> &(&'static Op, u32) {
        self.op_stack.last().unwrap()
    }

    fn pop_operator(&mut self) {
        let (op, pos) = self.op_stack.pop().unwrap();
        let t = Box::new(self.exp_stack.pop().unwrap());
        match op {
            &Op { ch, typ: OpType::Binary, .. } => {
                let t0 = Box::new(self.exp_stack.pop().unwrap());
                self.exp_stack.push(AstNode::new(Binary(ch, t0, t), pos));
            },
            &Op { ch, typ: OpType::Prefix , .. } => self.exp_stack.push(AstNode::new(Prefix(ch, t), pos)),
            &Op { ch, typ: OpType::Postfix, .. } => self.exp_stack.push(AstNode::new(Postfix(ch, t), pos)),
            &Op { typ: OpType::Sentinel, .. } => panic!("Unexpected Sentinel from position {:?} on operator stack", pos),
        }
    }

    fn push_operator(&mut self, op_pos: (&'static Op, u32)) {
        while has_greater_prec(self.top_operator().0, op_pos.0) {
           self.pop_operator();
        }
        self.op_stack.push(op_pos);
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Shunting yard parser as described here
///   https://www.engr.mun.ca/~theo/Misc/exp_parsing.htm
/// It parses the following grammar:
///   E --> P {B P}
///   P --> "(" E ")" | U P | P V | ident "(" P ")" | ident | number
///   B --> "+" | "-" | "*" | "/" | "^"
///   U --> "-"
///   V --> "!"
pub fn parse(text: &str) -> Result<AstNode, ParseError> {
    let mut lexer = Lexer::new(text);
    let next = try!(lexer.next_token());
    ShuntingYard {
        lexer: lexer,
        next: next,
        op_stack: {
            let mut op_stack = Vec::new();
            op_stack.push((&SENTINEL, 0));
            op_stack
        },
        exp_stack: Vec::new(),
    }.parse()
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::parse;

    #[test]
    fn test() {
        let text = "(3*x+4)!!- 5*2^x!^2+log(zy^2^3)--5";
        println!(".123456789.123456789.123456789.123456789");
        println!("{}", text);
        let ast_node = parse(text).unwrap();
        println!("{}", ast_node);
        println!("{:?}", ast_node);
    }
}

