use crate::{Imm, InstC, Instruction};
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone)]
pub(crate) struct Statement {
    line: u32,
    kind: StatementKind,
}

impl Statement {
    pub(crate) fn new(line: u32, kind: StatementKind) -> Self {
        Self { line, kind }
    }

    pub(crate) fn line(&self) -> u32 {
        self.line
    }

    pub(crate) fn kind(&self) -> &StatementKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StatementKind {
    Label(String),
    AtLabel(String),
    A(Imm),
    C(InstC),
}

impl fmt::Display for StatementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementKind::Label(label) => write!(f, "({})", label),
            StatementKind::AtLabel(label) => write!(f, "@{}", label),
            StatementKind::A(a) => fmt::Display::fmt(a, f),
            StatementKind::C(c) => fmt::Display::fmt(c, f),
        }
    }
}

impl StatementKind {
    pub(crate) fn is_inst(&self) -> bool {
        match self {
            Self::Label(_) => false,
            Self::AtLabel(_) | Self::A(_) | Self::C(_) => true,
        }
    }

    pub(crate) fn to_inst(&self, sym_tab: &HashMap<String, Imm>) -> Option<Instruction> {
        match self {
            Self::Label(_) => None,
            Self::AtLabel(label) => sym_tab.get(label).map(|a| Instruction::A(*a)),
            Self::C(c) => Some(Instruction::C(*c)),
            Self::A(a) => Some(Instruction::A(*a)),
        }
    }
}
