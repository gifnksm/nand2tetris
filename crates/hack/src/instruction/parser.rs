use super::{Comp, Dest, Imm, InstC, Instruction, Jump};
use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;

impl FromStr for Instruction {
    type Err = ParseInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let code = u16::from_str_radix(s, 2).map_err(|e| Self::Err::ParseInt(e, s.to_owned()))?;
        let inst = Instruction::decode(code)?;
        Ok(inst)
    }
}

impl Instruction {
    pub fn decode(code: u16) -> Result<Self, DecodeInstructionError> {
        let kind = u8::try_from((code & 0b1110_0000_0000_0000) >> 13).unwrap();
        match kind {
            x if x & 0b100 == 0 => Ok(Instruction::A(Imm(code))),
            0b111 => {
                let comp = u8::try_from((code & 0b0001_1111_1100_0000) >> 6).unwrap();
                let dest = u8::try_from((code & 0b0000_0000_0011_1000) >> 3).unwrap();
                let jump = u8::try_from(code & 0b0000_0000_0000_0111).unwrap();
                let comp = Comp::try_from(comp).map_err(DecodeInstructionError::InvalidComp)?;
                let dest = Dest::try_from(dest).map_err(DecodeInstructionError::InvalidDest)?;
                let jump = Jump::try_from(jump).map_err(DecodeInstructionError::InvalidJump)?;
                Ok(Instruction::C(InstC { comp, dest, jump }))
            }
            kind => Err(DecodeInstructionError::InvalidKind(kind)),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseInstructionError {
    #[error("invalid code: {}", _1)]
    ParseInt(ParseIntError, String),
    #[error(transparent)]
    Decode(#[from] DecodeInstructionError),
}

#[derive(Debug, Error)]
pub enum DecodeInstructionError {
    #[error("invalid kind: {:03b}", _0)]
    InvalidKind(u8),
    #[error("invalid comp: {:07b}", _0)]
    InvalidComp(u8),
    #[error("invalid dest: {:03b}", _0)]
    InvalidDest(u8),
    #[error("invalid jump: {:03b}", _0)]
    InvalidJump(u8),
}
