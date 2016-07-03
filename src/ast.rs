use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Debug)]
pub enum AstType {
    Number(f64),
    Ident(String),
    Func(String, Box<AstNode>),
    Binary(char, Box<AstNode>, Box<AstNode>),
    Prefix(char, Box<AstNode>),
    Postfix(char, Box<AstNode>),
    Parens(Box<AstNode>),
}
use self::AstType::*;

#[derive(Clone, Debug)]
pub struct AstNode {
    pub typ: AstType,
    pub pos: u32,
}

impl AstNode {
    pub fn new(typ: AstType, pos: u32) -> AstNode {
        AstNode { typ: typ, pos: pos }
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.typ {
            &Number(n) => n.fmt(f),
            &Ident(ref s) => s.fmt(f),
            &Func(ref s, ref arg) => f.write_fmt(format_args!("{}({})", s, arg)),
            &Binary(ch, ref arg1, ref arg2) => f.write_fmt(format_args!("{}{}{}", arg1, ch, arg2)),
            &Prefix(ch, ref arg) => f.write_fmt(format_args!("{}{}", ch, arg)),
            &Postfix(ch, ref arg) => f.write_fmt(format_args!("{}{}", arg, ch)),
            &Parens(ref arg) => f.write_fmt(format_args!("({})", arg)),
        }
    }
}
