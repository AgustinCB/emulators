use instruction::AddressingMode;
use {CpuError, CpuResult, Mos6502Cpu};

pub(crate) const ONE_TWO_COMPLEMENT: u8 = 0xff;

impl Mos6502Cpu {
    #[inline]
    pub(crate) fn check_alu_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { .. } => Ok(()),
            AddressingMode::ZeroPage { .. } => Ok(()),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(()),
            AddressingMode::IndexedIndirect { .. } => Ok(()),
            AddressingMode::IndirectIndexed { .. } => Ok(()),
            AddressingMode::Absolute { .. } => Ok(()),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(()),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode),
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
}
