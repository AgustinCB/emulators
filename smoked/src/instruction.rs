use cpu::{Cycles, Instruction as CpuInstruction};
use failure::Error;
use log::warn;

pub enum Instruction {
    Return,
    Constant(usize),
    Nil,
    True,
    False,
    Plus,
    Minus,
    Mult,
    Div,
    Not,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Noop,
    StringConcat,
    Syscall,
    SetGlobal(usize),
    GetGlobal(usize),
    SetLocal(usize),
    GetLocal(usize),
    JmpIfFalse(usize),
    Jmp(usize),
    Loop(usize),
    Call,
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
            6 => Instruction::Nil,
            7 => Instruction::True,
            8 => Instruction::False,
            9 => Instruction::Not,
            10 => Instruction::Equal,
            11 => Instruction::NotEqual,
            12 => Instruction::Greater,
            13 => Instruction::GreaterEqual,
            14 => Instruction::Less,
            15 => Instruction::LessEqual,
            16 => Instruction::StringConcat,
            17 => Instruction::Syscall,
            18 => Instruction::GetGlobal(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            19 => Instruction::SetGlobal(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            20 => Instruction::GetLocal(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            21 => Instruction::SetLocal(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            22 => Instruction::JmpIfFalse(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            23 => Instruction::Jmp(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            24 => Instruction::Loop(usize::from_be_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]]
            )),
            25 => Instruction::Call,
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
            Instruction::Nil => "NIL".to_owned(),
            Instruction::True => "TRUE".to_owned(),
            Instruction::False => "FALSE".to_owned(),
            Instruction::Not => "NOT".to_owned(),
            Instruction::Equal => "EQUAL".to_owned(),
            Instruction::NotEqual => "NOTEQUAL".to_owned(),
            Instruction::Less => "LESS".to_owned(),
            Instruction::LessEqual => "LESS_EQUAL".to_owned(),
            Instruction::Greater => "GREATER".to_owned(),
            Instruction::GreaterEqual => "GREATER_EQUAL".to_owned(),
            Instruction::StringConcat => "STRING_CONCAT".to_owned(),
            Instruction::Syscall => "SYSCALL".to_owned(),
            Instruction::GetGlobal(g) => format!("GET_GLOBAL {}", g),
            Instruction::SetGlobal(g) => format!("SET_GLOBAL {}", g),
            Instruction::GetLocal(g) => format!("GET_LOCAL {}", g),
            Instruction::SetLocal(g) => format!("SET_LOCAL {}", g),
            Instruction::JmpIfFalse(offset) => format!("JMP_IF_FALSE {}", offset),
            Instruction::Jmp(offset) => format!("JMP {}", offset),
            Instruction::Loop(offset) => format!("LOOP {}", offset),
            Instruction::Call => "CALL".to_owned(),
        }
    }
}
