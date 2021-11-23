use crate::{Comp, Dest, Imm, InstC, Instruction, Jump};
use std::{borrow::Cow, collections::HashMap, fmt};

#[derive(Debug, Clone)]
pub(crate) struct StatementWithLine {
    line: u32,
    data: Statement,
}

impl StatementWithLine {
    pub(crate) fn new(line: u32, data: Statement) -> Self {
        Self { line, data }
    }

    pub(crate) fn line(&self) -> u32 {
        self.line
    }

    pub(crate) fn data(&self) -> &Statement {
        &self.data
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Label(Label),
    AtLabel(Label),
    A(Imm),
    C(InstC),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Label {
    SP,
    LCL,
    ARG,
    THIS,
    THAT,
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    SCREEN,
    KBD,
    Other(String),
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Label(label) => write!(f, "({})", label),
            Statement::AtLabel(label) => write!(f, "@{}", label),
            Statement::A(a) => fmt::Display::fmt(a, f),
            Statement::C(c) => fmt::Display::fmt(c, f),
        }
    }
}

impl Statement {
    pub fn label(label: Label) -> Self {
        Self::Label(label)
    }

    pub fn at_label(label: Label) -> Self {
        Self::AtLabel(label)
    }

    pub fn a(imm: Imm) -> Self {
        Self::A(imm)
    }

    pub fn c(dest: Dest, comp: Comp, jump: Jump) -> Self {
        Self::C(InstC::new(dest, comp, jump))
    }

    pub(crate) fn is_inst(&self) -> bool {
        match self {
            Self::Label(_) => false,
            Self::AtLabel(_) | Self::A(_) | Self::C(_) => true,
        }
    }

    pub(crate) fn to_inst(&self, sym_tab: &HashMap<String, Imm>) -> Option<Instruction> {
        match self {
            Self::Label(_) => None,
            Self::AtLabel(label) => sym_tab.get(label.as_str()).map(|a| Instruction::A(*a)),
            Self::C(c) => Some(Instruction::C(*c)),
            Self::A(a) => Some(Instruction::A(*a)),
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl From<Cow<'_, str>> for Label {
    fn from(s: Cow<str>) -> Self {
        match s.as_ref() {
            "SP" => Self::SP,
            "LCL" => Self::LCL,
            "ARG" => Self::ARG,
            "THIS" => Self::THIS,
            "THAT" => Self::THAT,
            "R0" => Self::R0,
            "R1" => Self::R1,
            "R2" => Self::R2,
            "R3" => Self::R3,
            "R4" => Self::R4,
            "R5" => Self::R5,
            "R6" => Self::R6,
            "R7" => Self::R7,
            "R8" => Self::R8,
            "R9" => Self::R9,
            "R10" => Self::R10,
            "R11" => Self::R11,
            "R12" => Self::R12,
            "R13" => Self::R13,
            "R14" => Self::R14,
            "R15" => Self::R15,
            "SCREEN" => Self::SCREEN,
            "KBD" => Self::KBD,
            _ => Self::Other(s.into_owned()),
        }
    }
}

impl From<&'_ str> for Label {
    fn from(s: &str) -> Self {
        Label::from(Cow::Borrowed(s))
    }
}

impl From<String> for Label {
    fn from(s: String) -> Self {
        Label::from(Cow::Owned(s))
    }
}

impl Label {
    pub fn as_str(&self) -> &str {
        match self {
            Label::SP => "SP",
            Label::LCL => "LCL",
            Label::ARG => "ARG",
            Label::THIS => "THIS",
            Label::THAT => "THAT",
            Label::R0 => "R0",
            Label::R1 => "R1",
            Label::R2 => "R2",
            Label::R3 => "R3",
            Label::R4 => "R4",
            Label::R5 => "R5",
            Label::R6 => "R6",
            Label::R7 => "R7",
            Label::R8 => "R8",
            Label::R9 => "R9",
            Label::R10 => "R10",
            Label::R11 => "R11",
            Label::R12 => "R12",
            Label::R13 => "R13",
            Label::R14 => "R14",
            Label::R15 => "R15",
            Label::SCREEN => "SCREEN",
            Label::KBD => "KBD",
            Label::Other(s) => s,
        }
    }
}
