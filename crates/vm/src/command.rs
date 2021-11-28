pub use self::parser::*;
use hack::Imm;
use std::{borrow::Borrow, fmt};

mod parser;
mod translator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Push(Segment, u16),
    Pop(Segment, u16),
    Label(Label),
    Goto(Label),
    IfGoto(Label),
    Function(FuncName, u8),
    Call(FuncName, u8),
    Return,
}

impl Command {
    pub(crate) fn is_jump(&self) -> bool {
        matches!(
            self,
            Command::Goto(..) | Command::IfGoto(..) | Command::Call(..) | Command::Return
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

impl Segment {
    fn len(&self) -> u16 {
        match self {
            Self::Argument => u16::from(u8::MAX) + 1,
            Self::Local | Self::Static | Self::Constant | Self::This | Self::That => {
                Imm::MAX.value() + 1
            }
            Self::Pointer => 2,
            Self::Temp => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Ident(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncName(String);

impl Borrow<str> for FuncName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Ident> for FuncName {
    fn from(s: Ident) -> Self {
        Self(s.0)
    }
}

impl fmt::Display for FuncName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FuncName {
    const TOPLEVEL: &'static str = "$toplevel";

    pub(crate) fn toplevel() -> Self {
        Self(Self::TOPLEVEL.to_string())
    }

    pub(crate) fn is_toplevel(&self) -> bool {
        self.0 == Self::TOPLEVEL
    }

    pub(crate) fn bootstrap() -> Self {
        Self("$bootstrap".to_string())
    }

    pub(crate) fn entry_point() -> Self {
        Self("Sys.init".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);

impl Borrow<str> for Label {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Ident> for Label {
    fn from(s: Ident) -> Self {
        Self(s.0)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Label {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
