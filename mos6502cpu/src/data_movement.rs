use {Mos6502Cpu, CpuResult, CpuError};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub fn execute_asl(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_movement_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        let future_carry = value & 0x80 > 0;
        let answer = value << 1;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = future_carry;
        self.set_value_to_addressing_mode(addressing_mode, answer)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn check_data_movement_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Accumulator => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {AVAILABLE_MEMORY, Mos6502Cpu};

    #[test]
    fn it_should_execut_asl_with_no_flag() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x03;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Asl,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x06);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_execut_asl_setting_carry() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0xc0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Asl,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_execut_asl_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Asl,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }
}