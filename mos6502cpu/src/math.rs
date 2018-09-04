use {Mos6502Cpu, CpuError, CpuResult};
use bit_utils::two_complement;
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_adc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        self.execute_adc_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_adc_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)? as u16;
        let carry_as_u16 = self.registers.p.carry as u16;
        let answer = self.registers.a as u16 + value + carry_as_u16;
        self.update_zero_flag(answer as u8);
        self.update_negative_flag(answer as u8);
        self.update_carry_flag(answer);
        self.registers.p.overflow =
            self.calculate_overflow(self.registers.a, (value + carry_as_u16) as u8, answer as u8);
        self.registers.a = answer as u8;
        Ok(())
    }

    pub(crate) fn execute_cmp(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        self.execute_cmp_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_cmp_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = two_complement(self.get_value_from_addressing_mode(addressing_mode)?);
        let a = self.registers.a;
        self.compare(a, value);
        Ok(())
    }

    pub(crate) fn execute_cpx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_compare_address(addressing_mode)?;
        let value = two_complement(self.get_value_from_addressing_mode(addressing_mode)?);
        let x = self.registers.x;
        self.compare(x, value);
        Ok(())
    }

    pub(crate) fn execute_cpy(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_compare_address(addressing_mode)?;
        let value = two_complement(self.get_value_from_addressing_mode(addressing_mode)?);
        let y = self.registers.y;
        self.compare(y, value);
        Ok(())
    }

    pub(crate) fn execute_sbc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        self.execute_sbc_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_sbc_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let carry_as_u16 = !self.registers.p.carry as u16;
        let a_as_u16 = self.registers.a as u16;
        let operand = two_complement(value) as u16;
        let answer = a_as_u16 + operand - carry_as_u16;
        self.update_zero_flag(answer as u8);
        self.update_negative_flag(answer as u8);
        self.update_carry_flag(answer);
        self.registers.p.overflow =
            self.calculate_overflow(self.registers.a, operand as u8, answer as u8);
        self.registers.a = answer as u8;
        Ok(())
    }

    #[inline]
    fn calculate_overflow(&self, op1: u8, op2: u8, result: u8) -> bool {
        ((op1 & 0x80) > 0 && (op2 & 0x80) > 0 && (result & 0x80) == 0) ||
            ((op1 & 0x80) == 0 && (op2 & 0x80) == 0 && (result & 0x80) > 0)
    }

    #[inline]
    fn compare(&mut self, register: u8, value: u8) {
        let answer = register as u16 + value as u16;
        self.update_zero_flag(answer as u8);
        self.update_negative_flag(answer as u8);
        self.update_carry_flag(answer);
    }

    #[inline]
    fn check_compare_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
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
    fn it_should_adc_without_prev_carry_overflow_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x21;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x21 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_without_overflow_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x21;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x21 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x43);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_overflow_without_prev_carry_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x84);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_overflow_without_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x85);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_carry_without_prev_carry_overflow_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x84;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x84 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x08);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_carry_without_overflow_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x84;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x84 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x09);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_zero_without_prev_carry_overflow_carry_nor_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xbe },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_zero_without_overflow_carry_nor_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x41;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xbe },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cmp_on_same_values_but_not_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_only_carry_on_cmp_with_smaller_value_but_not_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x41 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_negative_on_cmp_with_bigger_value() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x43 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cpx_on_same_values_but_not_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpx,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_only_carry_on_cpx_with_smaller_value_but_not_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpx,
            addressing_mode: AddressingMode::Immediate { byte: 0x41 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_negative_on_cpx_with_bigger_value() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpx,
            addressing_mode: AddressingMode::Immediate { byte: 0x43 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cpy_on_same_values_but_not_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpy,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_only_carry_on_cpy_with_smaller_value_but_not_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpy,
            addressing_mode: AddressingMode::Immediate { byte: 0x41 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_negative_on_cpy_with_bigger_value() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpy,
            addressing_mode: AddressingMode::Immediate { byte: 0x43 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_subtract_with_carry_set() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x41);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_with_carry_clear() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x40);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_carry_and_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0xc0;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x40 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_overflow() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x80;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x7f },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.overflow);
    }
}