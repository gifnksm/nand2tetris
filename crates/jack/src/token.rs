use std::borrow::Cow;

pub use self::lexer::*;
use parse_display::{Display, FromStr};

mod lexer;

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum Token {
    #[display("{0}")]
    Keyword(Keyword),
    #[display("{0}")]
    Symbol(Symbol),
    #[display("{0}")]
    Int(u16),
    #[display("{0}")]
    String(String),
    #[display("{0}")]
    Ident(Ident),
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Keyword(_) => TokenKind::Keyword,
            Token::Symbol(_) => TokenKind::Symbol,
            Token::Int(_) => TokenKind::Int,
            Token::String(_) => TokenKind::String,
            Token::Ident(_) => TokenKind::Ident,
        }
    }

    pub fn to_cow_str(&self) -> Cow<str> {
        match self {
            Token::Keyword(keyword) => Cow::from(keyword.as_str()),
            Token::Symbol(symbol) => Cow::from(symbol.as_str()),
            Token::Int(int) => Cow::from(int.to_string()),
            Token::String(string) => Cow::from(string.as_str()),
            Token::Ident(ident) => Cow::from(ident.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
pub enum TokenKind {
    Keyword,
    Symbol,
    Int,
    String,
    Ident,
}

#[derive(Debug, Clone, PartialEq, Eq, Display, FromStr)]
pub enum Keyword {
    #[display("class")]
    Class,
    #[display("constructor")]
    Constructor,
    #[display("function")]
    Function,
    #[display("method")]
    Method,
    #[display("field")]
    Field,
    #[display("static")]
    Static,
    #[display("var")]
    Var,
    #[display("int")]
    Int,
    #[display("char")]
    Char,
    #[display("boolean")]
    Boolean,
    #[display("void")]
    Void,
    #[display("true")]
    True,
    #[display("false")]
    False,
    #[display("null")]
    Null,
    #[display("this")]
    This,
    #[display("let")]
    Let,
    #[display("do")]
    Do,
    #[display("if")]
    If,
    #[display("else")]
    Else,
    #[display("while")]
    While,
    #[display("return")]
    Return,
}

impl Keyword {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Class => "class",
            Self::Constructor => "constructor",
            Self::Function => "function",
            Self::Method => "method",
            Self::Field => "field",
            Self::Static => "static",
            Self::Var => "var",
            Self::Int => "int",
            Self::Char => "char",
            Self::Boolean => "boolean",
            Self::Void => "void",
            Self::True => "true",
            Self::False => "false",
            Self::Null => "null",
            Self::This => "this",
            Self::Let => "let",
            Self::Do => "do",
            Self::If => "if",
            Self::Else => "else",
            Self::While => "while",
            Self::Return => "return",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display, FromStr)]
pub enum Symbol {
    #[display("(")]
    OpenParen,
    #[display(")")]
    CloseParen,
    #[display("{{")]
    OpenBrace,
    #[display("}}")]
    CloseBrace,
    #[display("[")]
    OpenBracket,
    #[display("]")]
    CloseBracket,
    #[display(".")]
    Dot,
    #[display(",")]
    Comma,
    #[display(";")]
    Semicolon,
    #[display("+")]
    Plus,
    #[display("-")]
    Minus,
    #[display("*")]
    Star,
    #[display("/")]
    Slash,
    #[display("&")]
    Ampersand,
    #[display("|")]
    VertBar,
    #[display("<")]
    Less,
    #[display(">")]
    Greater,
    #[display("=")]
    Equal,
    #[display("~")]
    Tilde,
}

impl Symbol {
    pub fn as_str(&self) -> &str {
        match self {
            Self::OpenParen => "(",
            Self::CloseParen => ")",
            Self::OpenBrace => "{",
            Self::CloseBrace => "}",
            Self::OpenBracket => "[",
            Self::CloseBracket => "]",
            Self::Dot => ".",
            Self::Comma => ",",
            Self::Semicolon => ";",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Ampersand => "&",
            Self::VertBar => "|",
            Self::Less => "<",
            Self::Greater => ">",
            Self::Equal => "=",
            Self::Tilde => "~",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
#[display("{0}")]
pub struct Ident(String);

impl Ident {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
