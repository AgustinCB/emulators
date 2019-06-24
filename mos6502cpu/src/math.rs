use instruction::AddressingMode;
use {CpuError, CpuResult, Mos6502Cpu};

impl Mos6502Cpu {
    pub(crate) fn execute_adc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        if self.decimal_enabled && self.registers.p.decimal {
            self.execute_adc_decimal_unchecked(addressing_mode)
        } else {
            self.execute_adc_unchecked(addressing_mode)
        }
    }

    #[inline]
    pub(crate) fn execute_adc_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let (tmp, first_carry) = self.registers.a.overflowing_add(value);
        let (answer, second_carry) = tmp.overflowing_add(self.registers.p.carry as u8);
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = first_carry || second_carry;
        self.registers.p.overflow = self.calculate_overflow(self.registers.a, value, answer);
        self.registers.a = answer;
        Ok(())
    }

    #[inline]
    pub(crate) fn execute_adc_decimal_unchecked(
        &mut self,
        addressing_mode: &AddressingMode,
    ) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let a = self.registers.a;
        let carry_as_u8 = self.registers.p.carry as u8;
        let mut al = (a & 0x0f) + (value & 0x0f) + carry_as_u8;
        let mut ah = (a >> 4) + (value >> 4) + (al > 0x09) as u8;
        if al > 9 {
            al += 6
        };
        self.update_zero_flag(a.wrapping_add(value).wrapping_add(carry_as_u8));
        self.registers.p.negative = (ah & 0x08) > 0;
        self.registers.p.overflow = (((ah << 4) ^ a) & 0x80) > 0 && ((a ^ value) & 0x80) == 0;
        if ah > 9 {
            ah += 6
        };
        self.registers.p.carry = ah > 15;
        self.registers.a = (ah << 4) | (al & 0x0f);
        Ok(())
    }

    pub(crate) fn execute_cmp(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        self.execute_cmp_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_cmp_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let a = self.registers.a;
        self.compare(a, value);
        Ok(())
    }

    pub(crate) fn execute_cpx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_compare_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let x = self.registers.x;
        self.compare(x, value);
        Ok(())
    }

    pub(crate) fn execute_cpy(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_compare_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let y = self.registers.y;
        self.compare(y, value);
        Ok(())
    }

    pub(crate) fn execute_sbc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        if self.decimal_enabled && self.registers.p.decimal {
            self.execute_sbc_decimal_unchecked(addressing_mode)
        } else {
            self.execute_sbc_unchecked(addressing_mode)
        }
    }

    #[inline]
    pub(crate) fn execute_sbc_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        self.registers.a = self.sbc_logic(value);
        Ok(())
    }

    #[inline]
    pub(crate) fn execute_sbc_decimal_unchecked(
        &mut self,
        addressing_mode: &AddressingMode,
    ) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let a = self.registers.a;
        let mut al = (a & 0x0f)
            .wrapping_sub(value & 0x0f)
            .wrapping_sub(!self.registers.p.carry as u8);
        if (al & 0x10) > 0 {
            al -= 6
        };
        let mut ah = (a >> 4)
            .wrapping_sub(value >> 4)
            .wrapping_sub(((al & 0x10) > 0) as u8);
        if (ah & 0x10) > 0 {
            ah -= 6
        };
        self.sbc_logic(value);
        self.registers.a = (ah << 4) | (al & 0x0f);
        Ok(())
    }

    #[inline]
    fn sbc_logic(&mut self, value: u8) -> u8 {
        let (tmp, first_carry) = self.registers.a.overflowing_sub(value);
        let (answer, second_carry) = tmp.overflowing_sub(!self.registers.p.carry as u8);
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = !(first_carry || second_carry);
        self.registers.p.overflow =
            ((self.registers.a ^ answer) & 0x80) > 0 && ((self.registers.a ^ value) & 0x80) > 0;
        answer
    }

    #[inline]
    fn calculate_overflow(&self, op1: u8, op2: u8, result: u8) -> bool {
        ((op1 & 0x80) > 0 && (op2 & 0x80) > 0 && (result & 0x80) == 0)
            || ((op1 & 0x80) == 0 && (op2 & 0x80) == 0 && (result & 0x80) > 0)
    }

    #[inline]
    pub(crate) fn compare(&mut self, register: u8, value: u8) -> u8 {
        let (answer, carry) = register.overflowing_sub(value);
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = !carry;
        answer
    }

    #[inline]
    fn check_compare_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { .. } => Ok(()),
            AddressingMode::ZeroPage { .. } => Ok(()),
            AddressingMode::Absolute { .. } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {Mos6502Cpu, AVAILABLE_MEMORY};

    #[test]
    fn it_should_adc_without_prev_carry_overflow_carry_negative_nor_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x21;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x21 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_without_overflow_carry_negative_nor_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x21;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x21 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x43);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_overflow_without_prev_carry_carry_negative_nor_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x84);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_overflow_without_carry_negative_nor_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x85);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_carry_without_prev_carry_overflow_negative_nor_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x84;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x84 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x08);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_carry_without_overflow_negative_nor_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x84;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x84 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x09);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_zero_without_prev_carry_overflow_carry_nor_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xbe },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_zero_without_overflow_carry_nor_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x41;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xbe },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_normally_when_decimal_is_disabled() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::without_decimal(Box::new(m));
        cpu.registers.a = 0x61;
        cpu.registers.p.carry = false;
        cpu.registers.p.decimal = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xb0 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x11);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cmp_on_same_values_but_not_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cmp_on_with_zeroes_but_not_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x0 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_carry_on_cmp_with_smaller_value() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x41 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_negative_on_cmp_with_bigger_value() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cmp,
            addressing_mode: AddressingMode::Immediate { byte: 0x43 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cpx_on_same_values_but_not_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpx,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_carry_on_cpx_with_smaller_value() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpx,
            addressing_mode: AddressingMode::Immediate { byte: 0x41 },
        })
        .unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_negative_on_cpx_with_bigger_value() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpx,
            addressing_mode: AddressingMode::Immediate { byte: 0x43 },
        })
        .unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_zero_and_carry_on_cpy_on_same_values_but_not_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpy,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_carry_on_cpy_with_smaller_value() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpy,
            addressing_mode: AddressingMode::Immediate { byte: 0x41 },
        })
        .unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_set_negative_on_cpy_with_bigger_value() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cpy,
            addressing_mode: AddressingMode::Immediate { byte: 0x43 },
        })
        .unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_subtract_with_carry_set() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x41);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_with_carry_clear() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x40);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_carry_and_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0xc0;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x40 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_overflow() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x7f;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0xff },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_setting_overflow_on_frontier() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x0;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_wrapping_around() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x0;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_without_wrapping_around() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x00;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_doing_nothing() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x00;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_subtract_normally_when_decimal_is_disabled() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x4F;
        cpu.registers.p.carry = true;
        cpu.registers.p.decimal = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sbc,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x4E);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.overflow);
    }
}
