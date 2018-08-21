use super::cpu::{Cycles, Instruction};

#[derive(Clone)]
pub enum Mos6502Instruction {
    Nop,
    Brk,
}


impl Instruction<u8> for Mos6502Instruction {
    fn size(&self) -> u8 {
        match self {
            Mos6502Instruction::Nop => 1,
            Mos6502Instruction::Brk => 1,
        }
    }

    fn get_cycles(&self) -> Cycles {
        match self {
            Mos6502Instruction::Nop => Cycles::Single(4),
            Mos6502Instruction::Brk => Cycles::Single(4),
        }
    }
}


impl From<Vec<u8>> for Mos6502Instruction {
    #[inline]
    fn from(bytes: Vec<u8>) -> Mos6502Instruction {
        match bytes[0] {
            0x00 => Mos6502Instruction::Brk,
            c => {
                eprintln!("Unrecognized byte {}.", c);
                Mos6502Instruction::Nop
            },
        }
    }
}

impl ToString for Mos6502Instruction {
    fn to_string(&self) -> String {
        match self {
            Mos6502Instruction::Nop => String::from("NOP"),
            Mos6502Instruction::Brk => String::from("BRK"),
        }
    }
}