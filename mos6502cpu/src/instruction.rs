use std::fmt;
use super::failure::Fail;
use super::cpu::{Cycles, Instruction};

#[derive(Debug, Fail)]
enum Mos6502InstructionError {
    #[fail(display = "Invalid Addressing Mode {} for {}", addressing_mode, instruction_code)]
    InvalidAddressingMode {
        addressing_mode: AddressingMode,
        instruction_code: Mos6502InstructionCode,
    },
}

#[derive(Clone, Debug)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate { byte: u8 },
    ZeroPage { byte: u8 },
    Absolute { high_byte: u8, low_byte: u8 },
    Relative { byte: u8 },
    Indirect,
    ZeroPageIndexedX { byte: u8 },
    ZeroPageIndexedY,
    AbsoluteIndexedX { high_byte: u8, low_byte: u8 },
    AbsoluteIndexedY { high_byte: u8, low_byte: u8 },
    IndexedIndirect { byte: u8 },
    IndirectIndexed { byte: u8 },
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            AddressingMode::Implicit => String::from(""),
            AddressingMode::Accumulator => String::from("A"),
            AddressingMode::Immediate { byte } => format!("#{:x}", byte),
            AddressingMode::ZeroPage { byte } => format!("{:x}", byte),
            AddressingMode::Absolute { high_byte, low_byte } =>
                format!("{:x}{:x}", high_byte, low_byte),
            AddressingMode::Relative { byte } => format!("PC+{:x}", byte),
            AddressingMode::Indirect => String::from("(a)"),
            AddressingMode::ZeroPageIndexedX { byte } => format!("{:x},x", byte),
            AddressingMode::ZeroPageIndexedY => String::from("d,y"),
            AddressingMode::AbsoluteIndexedX { high_byte, low_byte } =>
                format!("{:x}{:x},x", high_byte, low_byte),
            AddressingMode::AbsoluteIndexedY { high_byte, low_byte } =>
                format!("{:x}{:x},y", high_byte, low_byte),
            AddressingMode::IndexedIndirect { byte } => format!("({},x)", byte),
            AddressingMode::IndirectIndexed { byte } => format!("({}),y", byte),
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub enum Mos6502InstructionCode {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Nop,
}

impl fmt::Display for Mos6502InstructionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Mos6502InstructionCode::Adc => String::from("ADC"),
            Mos6502InstructionCode::And => String::from("AND"),
            Mos6502InstructionCode::Asl => String::from("ASL"),
            Mos6502InstructionCode::Bcc => String::from("BCC"),
            Mos6502InstructionCode::Bcs => String::from("BCS"),
            Mos6502InstructionCode::Beq => String::from("BEQ"),
            Mos6502InstructionCode::Bit => String::from("BIT"),
            Mos6502InstructionCode::Bmi => String::from("BMI"),
            Mos6502InstructionCode::Bne => String::from("BNE"),
            Mos6502InstructionCode::Bpl => String::from("BPL"),
            Mos6502InstructionCode::Brk => String::from("BRK"),
            Mos6502InstructionCode::Nop => String::from("NOP"),
        };
        write!(f, "{}", s)
    }
}

struct Mos6502Instruction {
    instruction: Mos6502InstructionCode,
    addressing_mode: AddressingMode,
}

impl Mos6502Instruction {
    fn alu_size(&self) -> Result<u8, Mos6502InstructionError> {
        match self.addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(2),
            AddressingMode::ZeroPage { byte: _ } => Ok(2),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(2),
            AddressingMode::IndexedIndirect { byte: _ } => Ok(2),
            AddressingMode::IndirectIndexed { byte: _ } => Ok(2),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(3),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(3),
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } => Ok(3),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn data_movement_size(&self) -> Result<u8, Mos6502InstructionError> {
        match self.addressing_mode {
            AddressingMode::Accumulator => Ok(1),
            AddressingMode::ZeroPage { byte: _ } => Ok(2),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(2),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(3),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(3),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn alu_cycles(&self) -> Result<Cycles, Mos6502InstructionError> {
        match self.addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(single!(2)),
            AddressingMode::ZeroPage { byte: _ } => Ok(single!(3)),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(single!(4)),
            AddressingMode::IndexedIndirect { byte: _ } => Ok(single!(6)),
            AddressingMode::IndirectIndexed { byte: _ } => Ok(conditional!(5, 6)),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(single!(4)),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } =>
                Ok(conditional!(4, 5)),
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } =>
                Ok(conditional!(4, 5)),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn data_movement_cycles(&self) -> Result<Cycles, Mos6502InstructionError> {
        match self.addressing_mode {
            AddressingMode::Accumulator => Ok(single!(2)),
            AddressingMode::ZeroPage { byte: _ } => Ok(single!(5)),
            AddressingMode::ZeroPageIndexedX { byte: _ }=> Ok(single!(6)),
            AddressingMode::Absolute { low_byte: _, high_byte: _ }=> Ok(single!(6)),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ }=> Ok(single!(7)),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    #[inline]
    fn invalid_addressing_mode(&self) -> Mos6502InstructionError {
        Mos6502InstructionError::InvalidAddressingMode {
            addressing_mode: self.addressing_mode.clone(),
            instruction_code: self.instruction.clone(),
        }
    }
}

impl Instruction<u8, Mos6502InstructionError> for Mos6502Instruction {
    fn size(&self) -> Result<u8, Mos6502InstructionError> {
        match self.instruction {
            Mos6502InstructionCode::Adc => self.alu_size(),
            Mos6502InstructionCode::And => self.alu_size(),
            Mos6502InstructionCode::Asl => self.data_movement_size(),
            Mos6502InstructionCode::Bcc => Ok(2),
            Mos6502InstructionCode::Bcs => Ok(2),
            Mos6502InstructionCode::Beq => Ok(2),
            Mos6502InstructionCode::Bit => match self.addressing_mode {
                AddressingMode::ZeroPage { byte: _ } => Ok(2),
                AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Bmi => Ok(2),
            Mos6502InstructionCode::Bne => Ok(2),
            Mos6502InstructionCode::Bpl => Ok(2),
            Mos6502InstructionCode::Brk => Ok(1),
            Mos6502InstructionCode::Nop => Ok(1),
        }
    }

    fn get_cycles(&self) -> Result<Cycles, Mos6502InstructionError> {
        match self.instruction {
            Mos6502InstructionCode::Adc => self.alu_cycles(),
            Mos6502InstructionCode::And => self.alu_cycles(),
            Mos6502InstructionCode::Asl => self.data_movement_cycles(),
            Mos6502InstructionCode::Bcc => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bcs => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Beq => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bit => match self.addressing_mode {
                AddressingMode::ZeroPage { byte: _ } => Ok(single!(3)),
                AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Bmi => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bne => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bpl => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Brk => Ok(single!(7)),
            Mos6502InstructionCode::Nop => Ok(single!(2)),
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
            0x06 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x0A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::Accumulator,
            },
            0x0E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x10 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bpl,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x16 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x1E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x21 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0x24 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bit,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x25 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x29 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x2C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bit,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x2D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x30 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bmi,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x31 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x35 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x39 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x3D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x61 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0x65 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x69 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x6D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x71 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x75 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x79 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x7D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2]
                },
            },
            0x90 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bcc,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0xB0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bcs,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0xD0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bne,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0xF0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Beq,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
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
        format!("{} {}", self.instruction, self.addressing_mode)
    }
}