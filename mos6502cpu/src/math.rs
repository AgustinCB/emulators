use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_adc(&mut self, addressing_mode: AddressingMode) -> CpuResult {
        self.check_alu_address(&addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        let carry_as_u8 = self.registers.p.carry as u8;
        let new_a = (self.registers.a + value + carry_as_u8) as u16;
        self.registers.p.zero = new_a == 0;
        self.registers.p.carry = new_a > 0xff;
        self.registers.p.negative = new_a & 0x80 > 0;
        self.registers.p.overflow =
            self.calculate_overflow(self.registers.a, value + carry_as_u8, new_a as u8);
        self.registers.a = new_a as u8;
        Ok(())
    }

    #[inline]
    fn check_alu_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
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
    fn calculate_overflow(&self, op1: u8, op2: u8, result: u8) -> bool {
        ((op1 & 0x80) > 0 && (op2 & 0x80) > 0 && (result & 0x80) == 0) ||
            ((op1 & 0x80) == 0 && (op2 & 0x80) == 0 && (result & 0x80) > 0)
    }
}