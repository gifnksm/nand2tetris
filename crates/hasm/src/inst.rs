use derive_try_from_primitive::TryFromPrimitive;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    A(Imm),
    C(InstC),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::A(a) => fmt::Display::fmt(a, f),
            Instruction::C(c) => fmt::Display::fmt(c, f),
        }
    }
}

impl Instruction {
    pub fn encode(&self) -> u16 {
        match self {
            Instruction::A(a) => a.0,
            Instruction::C(c) => {
                0b1110_0000_0000_0000 | (c.comp as u16) << 6 | (c.dest as u16) << 3 | c.jump as u16
            }
        }
    }

    pub fn decode(code: u16) -> Option<Self> {
        if code & 0b1000_0000_0000_0000 != 0 {
            return Some(Instruction::A(Imm(code)));
        }
        if code & 0b1110_0000_0000_0000 != 0b1110_0000_0000_0000 {
            return None;
        }
        let comp = u8::try_from((code & 0b0001_1111_1100_0000) >> 6).unwrap();
        let dest = u8::try_from((code & 0b0000_0000_0011_1000) >> 3).unwrap();
        let jump = u8::try_from(code & 0b0000_0000_0000_0111).unwrap();
        let comp = Comp::try_from(comp).ok()?;
        let dest = Dest::try_from(dest).ok()?;
        let jump = Jump::try_from(jump).ok()?;
        Some(Instruction::C(InstC { comp, dest, jump }))
    }

    pub fn a(imm: Imm) -> Self {
        Instruction::A(imm)
    }

    pub fn c(dest: Dest, comp: Comp, jump: Jump) -> Self {
        Instruction::C(InstC { dest, comp, jump })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imm(u16);

impl Imm {
    pub const SP: Self = Imm(0);
    pub const LCL: Self = Imm(1);
    pub const ARG: Self = Imm(2);
    pub const THIS: Self = Imm(3);
    pub const THAT: Self = Imm(4);
    pub const R0: Self = Imm(0);
    pub const R1: Self = Imm(1);
    pub const R2: Self = Imm(2);
    pub const R3: Self = Imm(3);
    pub const R4: Self = Imm(4);
    pub const R5: Self = Imm(5);
    pub const R6: Self = Imm(6);
    pub const R7: Self = Imm(7);
    pub const R8: Self = Imm(8);
    pub const R9: Self = Imm(9);
    pub const R10: Self = Imm(10);
    pub const R11: Self = Imm(11);
    pub const R12: Self = Imm(12);
    pub const R13: Self = Imm(13);
    pub const R14: Self = Imm(14);
    pub const R15: Self = Imm(15);
    pub const SCREEN: Self = Imm(0x4000);
    pub const KBD: Self = Imm(0x6000);
    pub const MAX: Self = Imm(0x7fff);

    pub fn try_new(value: u16) -> Option<Self> {
        if value > Self::MAX.value() {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Imm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

impl From<u8> for Imm {
    fn from(n: u8) -> Self {
        Self(u16::from(n))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstC {
    dest: Dest,
    comp: Comp,
    jump: Jump,
}

impl InstC {
    pub fn new(dest: Dest, comp: Comp, jump: Jump) -> Self {
        Self { dest, comp, jump }
    }

    pub fn dest(&self) -> Dest {
        self.dest
    }

    pub fn comp(&self) -> Comp {
        self.comp
    }

    pub fn jump(&self) -> Jump {
        self.jump
    }
}

impl fmt::Display for InstC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dest != Dest::Null {
            write!(f, "{}=", self.dest)?;
        }
        write!(f, "{}", self.comp)?;
        if self.jump != Jump::Null {
            write!(f, ";{}", self.jump)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
#[allow(clippy::unusual_byte_groupings)]
pub enum Comp {
    Zero = 0b0_101_010,
    One = 0b0_111_111,
    MinusOne = 0b0_111_010,
    D = 0b0_001_100,
    A = 0b0_110_000,
    NotD = 0b0_001_101,
    NotA = 0b0_110_001,
    MinusD = 0b0_001_111,
    MinusA = 0b0_110_011,
    DPlusOne = 0b0_011_111,
    APlusOne = 0b0_110_111,
    DMinusOne = 0b0_001_110,
    AMinusOne = 0b0_110_010,
    DPlusA = 0b0_000_010,
    DMinusA = 0b0_010_011,
    AMinusD = 0b0_000_111,
    DAndA = 0b0_000_000,
    DOrA = 0b0_010_101,

    M = 0b1_110_000,
    NotM = 0b1_110_001,
    MinusM = 0b1_110_011,
    MPlusOne = 0b1_110_111,
    MMinusOne = 0b1_110_010,
    DPlusM = 0b1_000_010,
    DMinusM = 0b1_010_011,
    MMinusD = 0b1_000_111,
    DAndM = 0b1_000_000,
    DOrM = 0b1_010_101,
}

impl fmt::Display for Comp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Comp::Zero => "0",
            Comp::One => "1",
            Comp::MinusOne => "-1",
            Comp::D => "D",
            Comp::A => "A",
            Comp::NotD => "!D",
            Comp::NotA => "!A",
            Comp::MinusD => "-D",
            Comp::MinusA => "-A",
            Comp::DPlusOne => "D+1",
            Comp::APlusOne => "A+1",
            Comp::DMinusOne => "D-1",
            Comp::AMinusOne => "A-1",
            Comp::DPlusA => "D+A",
            Comp::DMinusA => "D-A",
            Comp::AMinusD => "A-D",
            Comp::DAndA => "D&A",
            Comp::DOrA => "D|A",
            Comp::M => "M",
            Comp::NotM => "!M",
            Comp::MinusM => "-M",
            Comp::MPlusOne => "M+1",
            Comp::MMinusOne => "M-1",
            Comp::DPlusM => "D+M",
            Comp::DMinusM => "D-M",
            Comp::MMinusD => "M-D",
            Comp::DAndM => "D&M",
            Comp::DOrM => "D|M",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Dest {
    Null = 0b000,
    M = 0b001,
    D = 0b010,
    MD = 0b011,
    A = 0b100,
    AM = 0b101,
    AD = 0b110,
    AMD = 0b111,
}

impl fmt::Display for Dest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Dest::Null => "",
            Dest::M => "M",
            Dest::D => "D",
            Dest::MD => "MD",
            Dest::A => "A",
            Dest::AM => "AM",
            Dest::AD => "AD",
            Dest::AMD => "AMD",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum Jump {
    Null = 0b000,
    Gt = 0b001,
    Eq = 0b010,
    Ge = 0b011,
    Lt = 0b100,
    Ne = 0b101,
    Le = 0b110,
    Jmp = 0b111,
}

impl fmt::Display for Jump {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Jump::Null => "",
            Jump::Gt => "JGT",
            Jump::Eq => "JEQ",
            Jump::Ge => "JGE",
            Jump::Lt => "JLT",
            Jump::Ne => "JNE",
            Jump::Le => "JLE",
            Jump::Jmp => "JMP",
        };
        write!(f, "{}", s)
    }
}
