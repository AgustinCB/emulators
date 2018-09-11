use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

pub(crate) const ONE_TWO_COMPLEMENT: u8 = 0xff;

impl Mos6502Cpu {
    #[inline]
    pub(crate) fn check_alu_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::IndexedIndirect { byte: _ } => Ok(()),
            AddressingMode::IndirectIndexed { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    pub(crate) fn update_zero_flag(&mut self, answer: u8) {
        self.registers.p.zero = answer == 0;
    }

    #[inline]
    pub(crate) fn update_negative_flag(&mut self, answer: u8) {
        self.registers.p.negative = answer & 0x80 > 0;
    }

    #[inline]
    pub(crate) fn update_carry_flag(&mut self, answer: u16) {
        self.registers.p.carry = answer > 0xff;
    }
}