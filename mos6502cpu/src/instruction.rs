use super::cpu::{Cycles, Instruction};
use super::failure::Error;
use std::fmt;

#[derive(Debug, Fail)]
pub enum Mos6502InstructionError {
    #[fail(
        display = "Invalid Addressing Mode {} for {}",
        addressing_mode, instruction_code
    )]
    InvalidAddressingMode {
        addressing_mode: AddressingMode,
        instruction_code: Mos6502InstructionCode,
    },
    #[fail(display = "Instruction {} doesn't have size", instruction_code)]
    NoSize {
        instruction_code: Mos6502InstructionCode,
    },
    #[fail(display = "Instruction {} doesn't have cycles", instruction_code)]
    NoCycles {
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
    Indirect { high_byte: u8, low_byte: u8 },
    ZeroPageIndexedX { byte: u8 },
    ZeroPageIndexedY { byte: u8 },
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
            AddressingMode::Immediate { byte } => format!("#{:02x}", byte),
            AddressingMode::ZeroPage { byte } => format!("${:02x}", byte),
            AddressingMode::Absolute {
                high_byte,
                low_byte,
            } => format!("${:02x}{:02x}", high_byte, low_byte),
            AddressingMode::Relative { byte } => format!("${:02x}", byte),
            AddressingMode::Indirect {
                high_byte,
                low_byte,
            } => format!("(${:02x}{:02x})", high_byte, low_byte),
            AddressingMode::ZeroPageIndexedX { byte } => format!("${:02x},x", byte),
            AddressingMode::ZeroPageIndexedY { byte } => format!("${:02x},y", byte),
            AddressingMode::AbsoluteIndexedX {
                high_byte,
                low_byte,
            } => format!("${:02x}{:02x},x", high_byte, low_byte),
            AddressingMode::AbsoluteIndexedY {
                high_byte,
                low_byte,
            } => format!("${:02x}{:02x},y", high_byte, low_byte),
            AddressingMode::IndexedIndirect { byte } => format!("(${:02x},x)", byte),
            AddressingMode::IndirectIndexed { byte } => format!("(${:02x}),y", byte),
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug)]
pub enum Mos6502InstructionCode {
    Adc,
    Ahx,
    Alr,
    Anc,
    And,
    Arr,
    Asl,
    Axs,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dcp,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Irq,
    Isc,
    Jmp,
    Jsr,
    Lax,
    Las,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nmi,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rla,
    Rol,
    Ror,
    Rra,
    Rst,
    Rti,
    Rts,
    Sax,
    Sbc,
    Sec,
    Sed,
    Sei,
    Shy,
    Shx,
    Slo,
    Sre,
    Sta,
    Stx,
    Sty,
    Tas,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
    Xaa,
}

impl fmt::Display for Mos6502InstructionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Mos6502InstructionCode::Adc => String::from("ADC"),
            Mos6502InstructionCode::Ahx => String::from("AHX"),
            Mos6502InstructionCode::Alr => String::from("ALR"),
            Mos6502InstructionCode::Anc => String::from("ANC"),
            Mos6502InstructionCode::And => String::from("AND"),
            Mos6502InstructionCode::Arr => String::from("ARR"),
            Mos6502InstructionCode::Asl => String::from("ASL"),
            Mos6502InstructionCode::Axs => String::from("AXS"),
            Mos6502InstructionCode::Bcc => String::from("BCC"),
            Mos6502InstructionCode::Bcs => String::from("BCS"),
            Mos6502InstructionCode::Beq => String::from("BEQ"),
            Mos6502InstructionCode::Bit => String::from("BIT"),
            Mos6502InstructionCode::Bmi => String::from("BMI"),
            Mos6502InstructionCode::Bne => String::from("BNE"),
            Mos6502InstructionCode::Bpl => String::from("BPL"),
            Mos6502InstructionCode::Brk => String::from("BRK"),
            Mos6502InstructionCode::Bvc => String::from("BVC"),
            Mos6502InstructionCode::Bvs => String::from("BVS"),
            Mos6502InstructionCode::Clc => String::from("CLC"),
            Mos6502InstructionCode::Cld => String::from("CLD"),
            Mos6502InstructionCode::Cli => String::from("CLI"),
            Mos6502InstructionCode::Clv => String::from("CLV"),
            Mos6502InstructionCode::Cmp => String::from("CMP"),
            Mos6502InstructionCode::Cpx => String::from("CPX"),
            Mos6502InstructionCode::Cpy => String::from("CPY"),
            Mos6502InstructionCode::Dcp => String::from("DCP"),
            Mos6502InstructionCode::Dec => String::from("DEC"),
            Mos6502InstructionCode::Dex => String::from("DEX"),
            Mos6502InstructionCode::Dey => String::from("DEY"),
            Mos6502InstructionCode::Eor => String::from("EOR"),
            Mos6502InstructionCode::Inc => String::from("INC"),
            Mos6502InstructionCode::Inx => String::from("INX"),
            Mos6502InstructionCode::Iny => String::from("INY"),
            Mos6502InstructionCode::Irq => String::from("IRQ"),
            Mos6502InstructionCode::Isc => String::from("ISC"),
            Mos6502InstructionCode::Jmp => String::from("JMP"),
            Mos6502InstructionCode::Jsr => String::from("JSR"),
            Mos6502InstructionCode::Las => String::from("LAS"),
            Mos6502InstructionCode::Lax => String::from("LAX"),
            Mos6502InstructionCode::Lda => String::from("LDA"),
            Mos6502InstructionCode::Ldx => String::from("LDX"),
            Mos6502InstructionCode::Ldy => String::from("LDY"),
            Mos6502InstructionCode::Lsr => String::from("LSR"),
            Mos6502InstructionCode::Nmi => String::from("NMI"),
            Mos6502InstructionCode::Nop => String::from("NOP"),
            Mos6502InstructionCode::Ora => String::from("ORA"),
            Mos6502InstructionCode::Pha => String::from("PHA"),
            Mos6502InstructionCode::Php => String::from("PHP"),
            Mos6502InstructionCode::Pla => String::from("PLA"),
            Mos6502InstructionCode::Plp => String::from("PLP"),
            Mos6502InstructionCode::Rla => String::from("RLA"),
            Mos6502InstructionCode::Rol => String::from("ROL"),
            Mos6502InstructionCode::Ror => String::from("ROR"),
            Mos6502InstructionCode::Rra => String::from("RRA"),
            Mos6502InstructionCode::Rst => String::from("RST"),
            Mos6502InstructionCode::Rti => String::from("RTI"),
            Mos6502InstructionCode::Rts => String::from("RTS"),
            Mos6502InstructionCode::Sax => String::from("SAX"),
            Mos6502InstructionCode::Sbc => String::from("SBC"),
            Mos6502InstructionCode::Sec => String::from("SEC"),
            Mos6502InstructionCode::Sed => String::from("SED"),
            Mos6502InstructionCode::Sei => String::from("SEI"),
            Mos6502InstructionCode::Shx => String::from("SHX"),
            Mos6502InstructionCode::Shy => String::from("SHY"),
            Mos6502InstructionCode::Slo => String::from("SLO"),
            Mos6502InstructionCode::Sre => String::from("SRE"),
            Mos6502InstructionCode::Sta => String::from("STA"),
            Mos6502InstructionCode::Stx => String::from("STX"),
            Mos6502InstructionCode::Sty => String::from("STY"),
            Mos6502InstructionCode::Tas => String::from("TAS"),
            Mos6502InstructionCode::Tax => String::from("TAX"),
            Mos6502InstructionCode::Tay => String::from("TAY"),
            Mos6502InstructionCode::Tsx => String::from("TSX"),
            Mos6502InstructionCode::Txa => String::from("TXA"),
            Mos6502InstructionCode::Txs => String::from("TXS"),
            Mos6502InstructionCode::Tya => String::from("TYA"),
            Mos6502InstructionCode::Xaa => String::from("XAA"),
        };
        write!(f, "{}", s)
    }
}

pub struct Mos6502Instruction {
    pub(crate) instruction: Mos6502InstructionCode,
    pub(crate) addressing_mode: AddressingMode,
}

impl Mos6502Instruction {
    pub fn new(
        instruction: Mos6502InstructionCode,
        addressing_mode: AddressingMode,
    ) -> Mos6502Instruction {
        Mos6502Instruction {
            instruction,
            addressing_mode,
        }
    }
    fn alu_size(&self) -> Result<u8, Error> {
        match self.addressing_mode {
            AddressingMode::Immediate { .. } => Ok(2),
            AddressingMode::ZeroPage { .. } => Ok(2),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
            AddressingMode::IndexedIndirect { .. } => Ok(2),
            AddressingMode::IndirectIndexed { .. } => Ok(2),
            AddressingMode::Absolute { .. } => Ok(3),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(3),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(3),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn alu_accumulator_size(&self) -> Result<u8, Error> {
        match self.addressing_mode {
            AddressingMode::ZeroPage { .. } => Ok(2),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
            AddressingMode::IndexedIndirect { .. } => Ok(2),
            AddressingMode::IndirectIndexed { .. } => Ok(2),
            AddressingMode::Absolute { .. } => Ok(3),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(3),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(3),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn data_movement_size(&self) -> Result<u8, Error> {
        match self.addressing_mode {
            AddressingMode::Accumulator => Ok(1),
            AddressingMode::ZeroPage { .. } => Ok(2),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
            AddressingMode::Absolute { .. } => Ok(3),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(3),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn alu_cycles(&self) -> Result<Cycles, Error> {
        match self.addressing_mode {
            AddressingMode::Immediate { .. } => Ok(single!(2)),
            AddressingMode::ZeroPage { .. } => Ok(single!(3)),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(4)),
            AddressingMode::IndexedIndirect { .. } => Ok(single!(6)),
            AddressingMode::IndirectIndexed { .. } => Ok(conditional!(5, 6)),
            AddressingMode::Absolute { .. } => Ok(single!(4)),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(conditional!(4, 5)),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(conditional!(4, 5)),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn alu_accumulator_cycles(&self) -> Result<Cycles, Error> {
        match self.addressing_mode {
            AddressingMode::ZeroPage { .. } => Ok(single!(3)),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(4)),
            AddressingMode::IndexedIndirect { .. } => Ok(single!(6)),
            AddressingMode::IndirectIndexed { .. } => Ok(single!(6)),
            AddressingMode::Absolute { .. } => Ok(single!(4)),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(single!(5)),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(single!(5)),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn unofficial_alu_accumulator_cycles(&self) -> Result<Cycles, Error> {
        match self.addressing_mode {
            AddressingMode::ZeroPage { .. } => Ok(single!(5)),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(6)),
            AddressingMode::IndexedIndirect { .. } => Ok(single!(8)),
            AddressingMode::IndirectIndexed { .. } => Ok(single!(8)),
            AddressingMode::Absolute { .. } => Ok(single!(6)),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(single!(7)),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(single!(7)),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    fn data_movement_cycles(&self) -> Result<Cycles, Error> {
        match self.addressing_mode {
            AddressingMode::Accumulator => Ok(single!(2)),
            AddressingMode::ZeroPage { .. } => Ok(single!(5)),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(6)),
            AddressingMode::Absolute { .. } => Ok(single!(6)),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(single!(7)),
            _ => Err(self.invalid_addressing_mode()),
        }
    }

    #[inline]
    fn invalid_addressing_mode(&self) -> Error {
        Error::from(Mos6502InstructionError::InvalidAddressingMode {
            addressing_mode: self.addressing_mode.clone(),
            instruction_code: self.instruction.clone(),
        })
    }
}

impl Instruction for Mos6502Instruction {
    fn size(&self) -> Result<u8, Error> {
        match self.instruction {
            Mos6502InstructionCode::Adc => self.alu_size(),
            Mos6502InstructionCode::Ahx => match self.addressing_mode {
                AddressingMode::AbsoluteIndexedY { .. } => Ok(3),
                AddressingMode::IndirectIndexed { .. } => Ok(2),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Alr => Ok(2),
            Mos6502InstructionCode::Anc => Ok(2),
            Mos6502InstructionCode::And => self.alu_size(),
            Mos6502InstructionCode::Arr => Ok(2),
            Mos6502InstructionCode::Asl => self.data_movement_size(),
            Mos6502InstructionCode::Axs => Ok(2),
            Mos6502InstructionCode::Bcc => Ok(2),
            Mos6502InstructionCode::Bcs => Ok(2),
            Mos6502InstructionCode::Beq => Ok(2),
            Mos6502InstructionCode::Bit => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Bmi => Ok(2),
            Mos6502InstructionCode::Bne => Ok(2),
            Mos6502InstructionCode::Bpl => Ok(2),
            Mos6502InstructionCode::Brk => Ok(1),
            Mos6502InstructionCode::Bvc => Ok(2),
            Mos6502InstructionCode::Bvs => Ok(2),
            Mos6502InstructionCode::Clc => Ok(1),
            Mos6502InstructionCode::Cld => Ok(1),
            Mos6502InstructionCode::Cli => Ok(1),
            Mos6502InstructionCode::Clv => Ok(1),
            Mos6502InstructionCode::Cmp => self.alu_size(),
            Mos6502InstructionCode::Cpx => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(2),
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Cpy => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(2),
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Dcp => self.alu_accumulator_size(),
            Mos6502InstructionCode::Dec => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Dex => Ok(1),
            Mos6502InstructionCode::Dey => Ok(1),
            Mos6502InstructionCode::Eor => self.alu_size(),
            Mos6502InstructionCode::Inc => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Inx => Ok(1),
            Mos6502InstructionCode::Iny => Ok(1),
            Mos6502InstructionCode::Irq => Err(Error::from(Mos6502InstructionError::NoSize {
                instruction_code: Mos6502InstructionCode::Irq,
            })),
            Mos6502InstructionCode::Isc => self.alu_accumulator_size(),
            Mos6502InstructionCode::Jmp => match self.addressing_mode {
                AddressingMode::Indirect { .. } => Ok(3),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Jsr => Ok(3),
            Mos6502InstructionCode::Las => Ok(3),
            Mos6502InstructionCode::Lax => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(2),
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(2),
                AddressingMode::IndexedIndirect { .. } => Ok(2),
                AddressingMode::IndirectIndexed { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                AddressingMode::AbsoluteIndexedY { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Lda => self.alu_size(),
            Mos6502InstructionCode::Ldx => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(2),
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                AddressingMode::AbsoluteIndexedY { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Ldy => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(2),
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Lsr => self.data_movement_size(),
            Mos6502InstructionCode::Nmi => Err(Error::from(Mos6502InstructionError::NoSize {
                instruction_code: Mos6502InstructionCode::Nmi,
            })),
            Mos6502InstructionCode::Nop => Ok(1),
            Mos6502InstructionCode::Ora => self.alu_size(),
            Mos6502InstructionCode::Pha => Ok(1),
            Mos6502InstructionCode::Php => Ok(1),
            Mos6502InstructionCode::Pla => Ok(1),
            Mos6502InstructionCode::Plp => Ok(1),
            Mos6502InstructionCode::Rla => self.alu_accumulator_size(),
            Mos6502InstructionCode::Rol => self.data_movement_size(),
            Mos6502InstructionCode::Ror => self.data_movement_size(),
            Mos6502InstructionCode::Rra => self.alu_accumulator_size(),
            Mos6502InstructionCode::Rst => Err(Error::from(Mos6502InstructionError::NoSize {
                instruction_code: Mos6502InstructionCode::Rst,
            })),
            Mos6502InstructionCode::Rti => Ok(1),
            Mos6502InstructionCode::Rts => Ok(1),
            Mos6502InstructionCode::Sax => match self.addressing_mode {
                AddressingMode::IndexedIndirect { .. } => Ok(2),
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Sbc => self.alu_size(),
            Mos6502InstructionCode::Sec => Ok(1),
            Mos6502InstructionCode::Sed => Ok(1),
            Mos6502InstructionCode::Sei => Ok(1),
            Mos6502InstructionCode::Shx => Ok(3),
            Mos6502InstructionCode::Shy => Ok(3),
            Mos6502InstructionCode::Slo => self.alu_accumulator_size(),
            Mos6502InstructionCode::Sre => self.alu_accumulator_size(),
            Mos6502InstructionCode::Sta => self.alu_accumulator_size(),
            Mos6502InstructionCode::Stx => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Sty => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(2),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(2),
                AddressingMode::Absolute { .. } => Ok(3),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Tas => Ok(3),
            Mos6502InstructionCode::Tax => Ok(1),
            Mos6502InstructionCode::Tay => Ok(1),
            Mos6502InstructionCode::Tsx => Ok(1),
            Mos6502InstructionCode::Txa => Ok(1),
            Mos6502InstructionCode::Txs => Ok(1),
            Mos6502InstructionCode::Tya => Ok(1),
            Mos6502InstructionCode::Xaa => Ok(2),
        }
    }

    fn get_cycles(&self) -> Result<Cycles, Error> {
        match self.instruction {
            Mos6502InstructionCode::Adc => self.alu_cycles(),
            Mos6502InstructionCode::Ahx => Ok(conditional!(5, 6)),
            Mos6502InstructionCode::Alr => Ok(single!(2)),
            Mos6502InstructionCode::Anc => Ok(single!(2)),
            Mos6502InstructionCode::And => self.alu_cycles(),
            Mos6502InstructionCode::Arr => Ok(single!(2)),
            Mos6502InstructionCode::Asl => self.data_movement_cycles(),
            Mos6502InstructionCode::Axs => Ok(single!(2)),
            Mos6502InstructionCode::Bcc => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bcs => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Beq => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bit => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Bmi => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bne => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bpl => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Brk => Ok(single!(7)),
            Mos6502InstructionCode::Bvc => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Bvs => Ok(bi_conditional!(2, 3, 5)),
            Mos6502InstructionCode::Clc => Ok(single!(2)),
            Mos6502InstructionCode::Cld => Ok(single!(2)),
            Mos6502InstructionCode::Cli => Ok(single!(2)),
            Mos6502InstructionCode::Clv => Ok(single!(2)),
            Mos6502InstructionCode::Cmp => self.alu_cycles(),
            Mos6502InstructionCode::Cpx => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(single!(2)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Cpy => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(single!(2)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Dec => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(single!(5)),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(6)),
                AddressingMode::Absolute { .. } => Ok(single!(6)),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(single!(7)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Dcp => self.unofficial_alu_accumulator_cycles(),
            Mos6502InstructionCode::Dex => Ok(single!(2)),
            Mos6502InstructionCode::Dey => Ok(single!(2)),
            Mos6502InstructionCode::Eor => self.alu_cycles(),
            Mos6502InstructionCode::Inc => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(single!(5)),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(6)),
                AddressingMode::Absolute { .. } => Ok(single!(6)),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(single!(7)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Inx => Ok(single!(2)),
            Mos6502InstructionCode::Iny => Ok(single!(2)),
            Mos6502InstructionCode::Irq => Ok(single!(7)),
            Mos6502InstructionCode::Isc => self.unofficial_alu_accumulator_cycles(),
            Mos6502InstructionCode::Jmp => match self.addressing_mode {
                AddressingMode::Indirect { .. } => Ok(single!(5)),
                AddressingMode::Absolute { .. } => Ok(single!(3)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Jsr => Ok(single!(6)),
            Mos6502InstructionCode::Las => Ok(conditional!(4, 5)),
            Mos6502InstructionCode::Lax => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(single!(2)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(single!(4)),
                AddressingMode::IndexedIndirect { .. } => Ok(single!(6)),
                AddressingMode::IndirectIndexed { .. } => Ok(conditional!(5, 6)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                AddressingMode::AbsoluteIndexedY { .. } => Ok(conditional!(4, 5)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Lda => self.alu_cycles(),
            Mos6502InstructionCode::Ldx => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(single!(2)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(single!(4)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                AddressingMode::AbsoluteIndexedY { .. } => Ok(conditional!(4, 5)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Ldy => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(single!(2)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(4)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(conditional!(4, 5)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Lsr => self.data_movement_cycles(),
            Mos6502InstructionCode::Nmi => Ok(single!(7)),
            Mos6502InstructionCode::Nop => match self.addressing_mode {
                AddressingMode::Immediate { .. } => Ok(single!(2)),
                AddressingMode::Implicit => Ok(single!(2)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(4)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                AddressingMode::AbsoluteIndexedX { .. } => Ok(conditional!(4, 5)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Ora => self.alu_cycles(),
            Mos6502InstructionCode::Pha => Ok(single!(3)),
            Mos6502InstructionCode::Php => Ok(single!(3)),
            Mos6502InstructionCode::Pla => Ok(single!(4)),
            Mos6502InstructionCode::Plp => Ok(single!(4)),
            Mos6502InstructionCode::Rla => self.unofficial_alu_accumulator_cycles(),
            Mos6502InstructionCode::Rol => self.data_movement_cycles(),
            Mos6502InstructionCode::Ror => self.data_movement_cycles(),
            Mos6502InstructionCode::Rra => self.unofficial_alu_accumulator_cycles(),
            Mos6502InstructionCode::Rst => Ok(single!(7)),
            Mos6502InstructionCode::Rti => Ok(single!(6)),
            Mos6502InstructionCode::Rts => Ok(single!(6)),
            Mos6502InstructionCode::Sax => match self.addressing_mode {
                AddressingMode::IndexedIndirect { .. } => Ok(single!(6)),
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(single!(4)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Sbc => self.alu_cycles(),
            Mos6502InstructionCode::Sec => Ok(single!(2)),
            Mos6502InstructionCode::Sed => Ok(single!(2)),
            Mos6502InstructionCode::Sei => Ok(single!(2)),
            Mos6502InstructionCode::Shy => Ok(single!(5)),
            Mos6502InstructionCode::Shx => Ok(single!(5)),
            Mos6502InstructionCode::Slo => self.unofficial_alu_accumulator_cycles(),
            Mos6502InstructionCode::Sre => self.unofficial_alu_accumulator_cycles(),
            Mos6502InstructionCode::Sta => self.alu_accumulator_cycles(),
            Mos6502InstructionCode::Stx => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedY { .. } => Ok(single!(4)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Sty => match self.addressing_mode {
                AddressingMode::ZeroPage { .. } => Ok(single!(3)),
                AddressingMode::ZeroPageIndexedX { .. } => Ok(single!(4)),
                AddressingMode::Absolute { .. } => Ok(single!(4)),
                _ => Err(self.invalid_addressing_mode()),
            },
            Mos6502InstructionCode::Tas => Ok(single!(5)),
            Mos6502InstructionCode::Tax => Ok(single!(2)),
            Mos6502InstructionCode::Tay => Ok(single!(2)),
            Mos6502InstructionCode::Tsx => Ok(single!(2)),
            Mos6502InstructionCode::Txa => Ok(single!(2)),
            Mos6502InstructionCode::Txs => Ok(single!(2)),
            Mos6502InstructionCode::Tya => Ok(single!(2)),
            Mos6502InstructionCode::Xaa => Ok(single!(2)),
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
            0x01 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0x04 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x05 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x06 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x08 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Php,
                addressing_mode: AddressingMode::Implicit,
            },
            0x09 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x0A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::Accumulator,
            },
            0x0C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x0D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x0E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x10 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bpl,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x11 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x14 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x15 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x16 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x18 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Clc,
                addressing_mode: AddressingMode::Implicit,
            },
            0x19 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x1A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
            0x1C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x1D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ora,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x1E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Asl,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x20 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Jsr,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
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
            0x26 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rol,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x28 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Plp,
                addressing_mode: AddressingMode::Implicit,
            },
            0x29 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x2A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rol,
                addressing_mode: AddressingMode::Accumulator,
            },
            0x2C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bit,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x2D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x2E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rol,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x30 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bmi,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x31 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x34 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x35 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x36 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rol,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x38 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sec,
                addressing_mode: AddressingMode::Implicit,
            },
            0x39 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x3A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
            0x3C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x3D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::And,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x3E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rol,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x40 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rti,
                addressing_mode: AddressingMode::Implicit,
            },
            0x41 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0x44 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x45 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x46 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lsr,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x48 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Pha,
                addressing_mode: AddressingMode::Implicit,
            },
            0x49 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x4A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lsr,
                addressing_mode: AddressingMode::Accumulator,
            },
            0x4C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Jmp,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x4D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x4E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lsr,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x50 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bvc,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x51 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x54 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x55 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x56 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lsr,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x58 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cli,
                addressing_mode: AddressingMode::Implicit,
            },
            0x59 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x5A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
            0x5C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x5D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Eor,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x5E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lsr,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x60 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Rts,
                addressing_mode: AddressingMode::Implicit,
            },
            0x61 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0x64 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x65 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x66 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ror,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x68 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Pla,
                addressing_mode: AddressingMode::Implicit,
            },
            0x69 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x6A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ror,
                addressing_mode: AddressingMode::Accumulator,
            },
            0x6C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Jmp,
                addressing_mode: AddressingMode::Indirect {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x6D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x6E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ror,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x70 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bvs,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x71 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x74 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x75 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x76 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ror,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x78 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sei,
                addressing_mode: AddressingMode::Implicit,
            },
            0x79 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x7A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
            0x7C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x7D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Adc,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x7E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ror,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x80 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x81 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0x82 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x84 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sty,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x85 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x86 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Stx,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0x88 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Dey,
                addressing_mode: AddressingMode::Implicit,
            },
            0x89 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0x8A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Txa,
                addressing_mode: AddressingMode::Implicit,
            },
            0x8C => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sty,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x8D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x8E => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Stx,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x90 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bcc,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0x91 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0x94 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sty,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x95 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0x96 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Stx,
                addressing_mode: AddressingMode::ZeroPageIndexedY { byte: bytes[1] },
            },
            0x98 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Tya,
                addressing_mode: AddressingMode::Implicit,
            },
            0x99 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0x9A => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Txs,
                addressing_mode: AddressingMode::Implicit,
            },
            0x9D => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sta,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xB0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bcs,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0xA0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldy,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xA1 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0xA2 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldx,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xA4 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldy,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xA5 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xA6 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldx,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xA8 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Tay,
                addressing_mode: AddressingMode::Implicit,
            },
            0xA9 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xAA => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Tax,
                addressing_mode: AddressingMode::Implicit,
            },
            0xAC => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldy,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xAD => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xAE => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldx,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xB1 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0xB4 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldy,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xB5 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xB6 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldx,
                addressing_mode: AddressingMode::ZeroPageIndexedY { byte: bytes[1] },
            },
            0xB8 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Clv,
                addressing_mode: AddressingMode::Implicit,
            },
            0xB9 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xBA => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Tsx,
                addressing_mode: AddressingMode::Implicit,
            },
            0xBC => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldy,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xBD => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Lda,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xBE => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Ldx,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xC0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cpy,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xC1 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0xC2 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xC4 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cpy,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xC5 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xC6 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Dec,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xC8 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Iny,
                addressing_mode: AddressingMode::Implicit,
            },
            0xC9 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xCA => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Dex,
                addressing_mode: AddressingMode::Implicit,
            },
            0xCC => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cpy,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xCD => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xCE => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Dec,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xD0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Bne,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0xD1 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0xD4 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xD5 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xD6 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Dec,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xD8 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cld,
                addressing_mode: AddressingMode::Implicit,
            },
            0xD9 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xDA => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
            0xDC => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xDD => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cmp,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xDE => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Dec,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xE0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cpx,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xE1 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::IndexedIndirect { byte: bytes[1] },
            },
            0xE2 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xE4 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cpx,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xE5 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xE6 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Inc,
                addressing_mode: AddressingMode::ZeroPage { byte: bytes[1] },
            },
            0xE8 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Inx,
                addressing_mode: AddressingMode::Implicit,
            },
            0xE9 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xEB => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::Immediate { byte: bytes[1] },
            },
            0xEC => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Cpx,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xED => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xEE => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Inc,
                addressing_mode: AddressingMode::Absolute {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xF0 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Beq,
                addressing_mode: AddressingMode::Relative { byte: bytes[1] },
            },
            0xF1 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::IndirectIndexed { byte: bytes[1] },
            },
            0xF4 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xF5 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xF6 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Inc,
                addressing_mode: AddressingMode::ZeroPageIndexedX { byte: bytes[1] },
            },
            0xF8 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sed,
                addressing_mode: AddressingMode::Implicit,
            },
            0xF9 => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::AbsoluteIndexedY {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xFA => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::Implicit,
            },
            0xFC => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Nop,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xFD => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Sbc,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
            },
            0xFE => Mos6502Instruction {
                instruction: Mos6502InstructionCode::Inc,
                addressing_mode: AddressingMode::AbsoluteIndexedX {
                    low_byte: bytes[1],
                    high_byte: bytes[2],
                },
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
