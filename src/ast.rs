use std::fmt::{Debug, Display, Error, Formatter};

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

#[derive(Clone)]
pub struct AstNode {
    pub typ: AstType,
    pub pos: u32,
}

impl AstNode {
    pub fn new(typ: AstType, pos: u32) -> AstNode {
        AstNode { typ: typ, pos: pos }
    }

    pub fn as_ascii_math(&self) -> AsciiMathFmt { AsciiMathFmt(self) }
    pub fn as_tree(&self) -> TreeFmt { TreeFmt(self) }
}

impl Debug for AstNode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.as_tree().fmt(f)
    }
}

impl Display for AstNode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.as_ascii_math().fmt(f)
    }
}

////////////////////////////////////////////////////////////////////////////////
// AsciiMath output

#[derive(Copy, Clone)]
pub struct AsciiMathFmt<'a>(&'a AstNode);

impl<'a> Display for AsciiMathFmt<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match &self.0.typ {
            &Number(n) => Display::fmt(&n, f),
            &Ident(ref s) => Display::fmt(&s, f),
            &Func(ref s, ref arg) => f.write_fmt(format_args!("{}({})", s, arg)),
            &Binary(ch, ref arg1, ref arg2) => f.write_fmt(format_args!("{}{}{}", arg1, ch, arg2)),
            &Prefix(ch, ref arg) => f.write_fmt(format_args!("{}{}", ch, arg)),
            &Postfix(ch, ref arg) => f.write_fmt(format_args!("{}{}", arg, ch)),
            &Parens(ref arg) => f.write_fmt(format_args!("({})", arg)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tree output

#[derive(Copy, Clone)]
pub struct TreeFmt<'a>(&'a AstNode);

impl<'a> TreeFmt<'a> {
    fn format(&self, f: &mut Formatter, indent: usize) -> Result<(), Error> {
        const INDENT: usize = 2;

        // Output the line position and the indent.
        try!(f.write_fmt(format_args!("{:3}:{:width$} ", self.0.pos, "", width=indent)));
        match &self.0.typ {
            &Number(n) => f.write_fmt(format_args!("{}\n", n)),
            &Ident(ref s) => f.write_fmt(format_args!("{}\n", s)),
            &Func(ref s, ref arg) => {
                try!(f.write_fmt(format_args!("{}()\n", s)));
                arg.as_tree().format(f, indent + INDENT)
            }
            &Binary(ch, ref arg1, ref arg2) => {
                try!(f.write_fmt(format_args!("{}\n", ch)));
                try!(arg1.as_tree().format(f, indent + INDENT));
                arg2.as_tree().format(f, indent + INDENT)
            }
            &Prefix(ch, ref arg) => {
                try!(f.write_fmt(format_args!("{} (prefix)\n", ch)));
                arg.as_tree().format(f, indent + INDENT)
            }
            &Postfix(ch, ref arg) => {
                try!(f.write_fmt(format_args!("{} (postfix)\n", ch)));
                arg.as_tree().format(f, indent + INDENT)
            }
            &Parens(ref arg) => {
                try!(Display::fmt("()\n", f));
                arg.as_tree().format(f, indent + INDENT)
            }
        }
    }
}

impl<'a> Display for TreeFmt<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        self.format(f, 0)
    }
}
