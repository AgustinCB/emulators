use cpu::{Cycles, Instruction as CpuInstruction};
use failure::Error;
use log::warn;

pub enum Instruction {
    Return,
    Constant(usize),
    Plus,
    Minus,
    Mult,
    Div,
    Noop,
}

impl CpuInstruction for Instruction {
    fn size(&self) -> Result<u8, Error> {
        Ok(match self {
            Instruction::Constant(_) => 2,
            _ => 1,
        })
    }

    fn get_cycles(&self) -> Result<Cycles, Error> {
        Ok(Cycles::Single(1))
    }
}

impl From<Vec<u8>> for Instruction {
    #[inline]
    fn from(bytes: Vec<u8>) -> Instruction {
        match bytes[0] {
            0 => Instruction::Return,
            1 => Instruction::Constant(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            2 => Instruction::Plus,
            3 => Instruction::Minus,
            4 => Instruction::Mult,
            5 => Instruction::Div,
            255 => Instruction::Noop,
            _ => {
                warn!("Invalid instruction");
                Instruction::Noop
            },
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Return => "RETURN".to_owned(),
            Instruction::Constant(b) => format!("CONSTANT {}", b),
            Instruction::Plus => "PLUS".to_owned(),
            Instruction::Minus => "MINUS".to_owned(),
            Instruction::Mult => "MULT".to_owned(),
            Instruction::Div => "DIV".to_owned(),
            Instruction::Noop => "NOOP".to_owned(),
        }
    }
}
