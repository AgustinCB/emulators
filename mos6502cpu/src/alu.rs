use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

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
}