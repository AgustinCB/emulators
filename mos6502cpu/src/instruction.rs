use super::failure::Fail;
use super::cpu::{Cycles, Instruction};

#[derive(Debug, Fail)]
enum Mos6502InstructionError {
    #[fail(display = "Invalid Addressing Mode")]
    InvalidAddressingMode,
}

pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    Absolute,
    Relative,
    Indirect,
    ZeroPageIndexedX,
    ZeroPageIndexedY,
    AbsoluteIndexedX,
    AbsoluteIndexedY,
    IndexedIndirect,
    IndirectIndexed,
}

impl ToString for AddressingMode {
    fn to_string(&self) -> String {
        match self {
            AddressingMode::Implicit => String::from(""),
            AddressingMode::Accumulator => String::from("A"),
            AddressingMode::Immediate => String::from("#v"),
            AddressingMode::ZeroPage => String::from("d"),
            AddressingMode::Absolute => String::from("a"),
            AddressingMode::Relative => String::from("label"),
            AddressingMode::Indirect => String::from("(a)"),
            AddressingMode::ZeroPageIndexedX => String::from("d,x"),
            AddressingMode::ZeroPageIndexedY => String::from("d,y"),
            AddressingMode::AbsoluteIndexedX => String::from("a,x"),
            AddressingMode::AbsoluteIndexedY => String::from("a,y"),
            AddressingMode::IndexedIndirect => String::from("(d,x)"),
            AddressingMode::IndirectIndexed => String::from("(d),y"),
        }
    }
}

#[derive(Clone)]
pub enum Mos6502InstructionCode {
    Nop,
    Brk,
}

struct Mos6502Instruction {
    instruction: Mos6502InstructionCode,
    addressing_mode: AddressingMode,
}

impl Instruction<u8, Mos6502InstructionError> for Mos6502Instruction {
    fn size(&self) -> u8 {
        match self.instruction {
            Mos6502InstructionCode::Nop => 1,
            Mos6502InstructionCode::Brk => 1,
        }
    }

    fn get_cycles(&self) -> Result<Cycles, Mos6502InstructionError> {
        match self.instruction {
            Mos6502InstructionCode::Nop => Ok(Cycles::Single(2)),
            Mos6502InstructionCode::Brk => Ok(Cycles::Single(7)),
        }
    }
}


impl From<Vec<u8>> for Mos6502Instruction {
    #[inline]
    fn from(bytes: Vec<u8>) -> Mos6502Instruction {
        match bytes[0] {
            0x00 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Brk,
                addressing_mode: AddressingMode::Implicit,
            },
            _ => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
        }
    }
}

impl ToString for Mos6502Instruction {
    fn to_string(&self) -> String {
        match self.instruction {
            Mos6502InstructionCode::Nop => String::from("NOP"),
            Mos6502InstructionCode::Brk => String::from("BRK"),
        }
    }
}