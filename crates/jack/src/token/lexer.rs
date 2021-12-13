use super::{Ident, Keyword, Symbol, Token};
use std::{
    fmt,
    io::{self, prelude::*},
    num,
    str::FromStr,
};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct WithLoc<T> {
    pub data: T,
    pub loc: Location,
}

impl<T> WithLoc<T> {
    pub fn as_ref(&self) -> WithLoc<&T> {
        WithLoc {
            data: &self.data,
            loc: self.loc,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> WithLoc<U> {
        WithLoc {
            data: f(self.data),
            loc: self.loc,
        }
    }

    pub fn filter_map<U>(self, f: impl FnOnce(T) -> Option<U>) -> Option<WithLoc<U>> {
        f(self.data).map(|data| WithLoc {
            data,
            loc: self.loc,
        })
    }
}

impl<T, E> WithLoc<Result<T, E>> {
    pub fn transpose(self) -> Result<WithLoc<T>, WithLoc<E>> {
        match self.data {
            Ok(data) => Ok(WithLoc {
                data,
                loc: self.loc,
            }),
            Err(data) => Err(WithLoc {
                data,
                loc: self.loc,
            }),
        }
    }

    pub fn transpose_ok(self) -> Result<WithLoc<T>, E> {
        let data = self.data?;
        Ok(WithLoc {
            data,
            loc: self.loc,
        })
    }
}

impl<T> WithLoc<Option<T>> {
    pub fn transpose(self) -> Option<WithLoc<T>> {
        self.data.map(|data| WithLoc {
            data,
            loc: self.loc,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line_num: u32,
    pub column: u32,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line_num, self.column)
    }
}

impl Location {
    pub(crate) const fn builtin() -> Location {
        Self {
            line_num: 0,
            column: 0,
        }
    }
}

#[derive(Debug)]
pub struct Tokens<R> {
    reader: R,
    line_buf: String,
    line_num: u32,
    line_index: usize,
    in_multiline_comment: bool,
}

impl<R> Tokens<R> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            reader,
            line_buf: String::new(),
            line_num: 0,
            line_index: 0,
            in_multiline_comment: false,
        }
    }
}

impl<R> Iterator for Tokens<R>
where
    R: BufRead,
{
    type Item = Result<WithLoc<Token>, ParseTokenError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
            .map_err(|e| ParseTokenError {
                loc: Location {
                    line_num: self.line_num,
                    column: self.line_index as u32,
                },
                kind: e,
            })
            .transpose()
    }
}

fn split_start_matches(s: &str, pat: impl FnMut(char) -> bool) -> Option<&str> {
    let rest = s.trim_start_matches(pat);
    let match_end = s.len() - rest.len();
    if match_end == 0 {
        return None;
    }
    Some(&s[0..match_end])
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_numeric()
}

impl<R> Tokens<R>
where
    R: BufRead,
{
    fn next_token(&mut self) -> Result<Option<WithLoc<Token>>, ParseTokenErrorKind> {
        loop {
            if self.line_index == self.line_buf.len() {
                self.line_num += 1;
                self.line_buf.clear();
                if self.reader.read_line(&mut self.line_buf)? == 0 {
                    return Ok(None);
                }
                self.line_index = 0;
            }

            if self.skip_spaces_or_comments() {
                continue;
            }
            let loc = Location {
                line_num: self.line_num,
                column: self.line_index as u32,
            };
            if let Some(token) = self.keyword_or_ident() {
                return Ok(Some(WithLoc { data: token, loc }));
            }
            if let Some(token) = self.integer()? {
                return Ok(Some(WithLoc { data: token, loc }));
            }
            if let Some(token) = self.string()? {
                return Ok(Some(WithLoc { data: token, loc }));
            }
            if let Some(token) = self.symbol()? {
                return Ok(Some(WithLoc { data: token, loc }));
            }
            let ch = self.line().chars().next().unwrap();
            return Err(ParseTokenErrorKind::UnexpectedChar(ch));
        }
    }
}

impl<R> Tokens<R> {
    fn line(&self) -> &str {
        &self.line_buf[self.line_index..]
    }

    fn eat_matches(&mut self, pat: impl FnMut(char) -> bool) -> Option<&str> {
        let eaten = split_start_matches(&self.line_buf[self.line_index..], pat)?;
        self.line_index += eaten.len();
        Some(eaten)
    }

    fn skip_spaces_or_comments(&mut self) -> bool {
        let mut skipped = false;
        if self.in_multiline_comment {
            if let Some(end_index) = self.line().find("*/") {
                self.line_index += end_index + "*/".len();
                self.in_multiline_comment = false;
                return true;
            }
            self.line_index = self.line_buf.len();
            return true;
        }

        if self
            .eat_matches(|ch| char::is_ascii_whitespace(&ch))
            .is_some()
        {
            skipped = true;
        }

        if self.line().starts_with("//") {
            self.line_index = self.line_buf.len();
            return true;
        }

        if self.line().starts_with("/*") {
            self.line_index += "/*".len();
            self.in_multiline_comment = true;
            return true;
        }

        skipped
    }

    fn keyword_or_ident(&mut self) -> Option<Token> {
        if self.line().starts_with(is_identifier_start) {
            let ident = self.eat_matches(is_identifier_continue).unwrap();
            if let Ok(keyword) = Keyword::from_str(ident) {
                return Some(Token::Keyword(keyword));
            }
            return Some(Token::Ident(Ident(ident.into())));
        }
        None
    }

    fn integer(&mut self) -> Result<Option<Token>, ParseTokenErrorKind> {
        if let Some(integer) = self.eat_matches(|ch| ch.is_numeric()) {
            let n = u16::from_str(integer)
                .map_err(|e| ParseTokenErrorKind::Integer(e, integer.into()))?;
            if n > 32767 {
                return Err(ParseTokenErrorKind::IntegerOverflow(n));
            }
            return Ok(Some(Token::Int(n)));
        }
        Ok(None)
    }

    fn string(&mut self) -> Result<Option<Token>, ParseTokenErrorKind> {
        if self.line().starts_with('"') {
            self.line_index += '"'.len_utf8();
            let string = self.eat_matches(|ch| ch != '"').unwrap_or("").to_string();
            if !self.line().starts_with('"') {
                return Err(ParseTokenErrorKind::UnterminatedString);
            }
            self.line_index += '"'.len_utf8();
            return Ok(Some(Token::String(string)));
        }
        Ok(None)
    }

    fn symbol(&mut self) -> Result<Option<Token>, ParseTokenErrorKind> {
        if let Some(ch) = self.line().chars().next() {
            let ch_len = ch.len_utf8();
            let s = &self.line()[..ch_len];
            if let Ok(sym) = Symbol::from_str(s) {
                self.line_index += ch_len;
                return Ok(Some(Token::Symbol(sym)));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Error)]
#[error("parse token error at {}:{}", loc.line_num, loc.column)]
pub struct ParseTokenError {
    loc: Location,
    #[source]
    kind: ParseTokenErrorKind,
}

#[derive(Debug, Error)]
pub enum ParseTokenErrorKind {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("invalid integer: {}", _1)]
    Integer(#[source] num::ParseIntError, String),
    #[error("integer is too large: {}", _0)]
    IntegerOverflow(u16),
    #[error("unterminated string")]
    UnterminatedString,
    #[error("unexpected char: {}", _0)]
    UnexpectedChar(char),
}
