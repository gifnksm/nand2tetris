use super::*;
use std::fmt;

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => write!(f, "add"),
            Self::Sub => write!(f, "sub"),
            Self::Neg => write!(f, "neg"),
            Self::Eq => write!(f, "eq"),
            Self::Gt => write!(f, "gt"),
            Self::Lt => write!(f, "lt"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Not => write!(f, "not"),
            Self::Push(seg, idx) => write!(f, "push {} {}", seg, idx),
            Self::Pop(seg, idx) => write!(f, "pop {} {}", seg, idx),
            Self::Label(label) => write!(f, "label {}", label),
            Self::Goto(label) => write!(f, "goto {}", label),
            Self::IfGoto(label) => write!(f, "if-goto {}", label),
            Self::Function(func, arity) => write!(f, "function {} {}", func, arity),
            Self::Call(func, arity) => write!(f, "call {} {}", func, arity),
            Self::Return => write!(f, "return"),
        }
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Display for FuncName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
