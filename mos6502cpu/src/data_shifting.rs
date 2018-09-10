use {Mos6502Cpu, CpuResult, CpuError};
use instruction::AddressingMode;

const ONE_TWO_COMPLEMENT: u8 = 0xff;

impl Mos6502Cpu {
    pub(crate) fn execute_asl(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_shifting_address(addressing_mode)?;
        self.execute_asl_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_asl_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let future_carry = value & 0x80 > 0;
        let answer = value << 1;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = future_carry;
        self.set_value_to_addressing_mode(addressing_mode, answer)
    }

    pub(crate) fn execute_dec(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_memory_data_shifting_address(addressing_mode)?;
        self.execute_dec_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_dec_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = self.decrement(value);
        self.set_value_to_addressing_mode(addressing_mode, answer)
    }

    pub(crate) fn execute_dex(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let value = self.registers.x;
            let answer = self.decrement(value);
            self.registers.x = answer;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_dey(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let value = self.registers.y;
            let answer = self.decrement(value);
            self.registers.y = answer;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_inc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_memory_data_shifting_address(addressing_mode)?;
        self.execute_inc_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_inc_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = self.increment(value);
        self.set_value_to_addressing_mode(addressing_mode, answer)?;
        Ok(())
    }

    pub(crate) fn execute_inx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let value = self.registers.x;
            let answer = self.increment(value);
            self.registers.x = answer;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_iny(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let value = self.registers.y;
            let answer = self.increment(value);
            self.registers.y = answer;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_lsr(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_shifting_address(addressing_mode)?;
        self.execute_lsr_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_lsr_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = value >> 1;
        self.update_zero_flag(answer);
        self.registers.p.carry = value & 0x01 > 0;
        self.registers.p.negative = false;
        self.set_value_to_addressing_mode(addressing_mode, answer)
    }

    pub(crate) fn execute_rol(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_shifting_address(addressing_mode)?;
        self.execute_rol_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_rol_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let carry_mask = self.registers.p.carry as u8;
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = (value << 1) | carry_mask;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = value & 0x80 > 0;
        self.set_value_to_addressing_mode(addressing_mode, answer)
    }

    pub(crate) fn execute_ror(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_shifting_address(addressing_mode)?;
        self.execute_ror_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_ror_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let carry_mask = (self.registers.p.carry as u8) << 7;
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = (value >> 1) | carry_mask;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        self.registers.p.carry = value & 0x01 > 0;
        self.set_value_to_addressing_mode(addressing_mode, answer)
    }

    #[inline]
    fn increment(&mut self, value: u8) -> u8 {
        let answer = value.wrapping_add(1);
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        answer
    }

    #[inline]
    fn decrement(&mut self, value: u8) -> u8 {
        let answer = value.wrapping_add(ONE_TWO_COMPLEMENT);
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        answer
    }

    #[inline]
    fn check_data_shifting_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Accumulator => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn check_memory_data_shifting_address(
        &self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
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

    #[test]
    fn it_should_substract_one_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dec,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x41);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x01;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dec,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x0);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dec,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0xff);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_from_x_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dex,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x41);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_from_x_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x01;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dex,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x0);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_from_x_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dex,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0xff);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_from_y_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dey,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x41);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_from_y_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x01;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dey,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x0);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_substract_one_from_y_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Dey,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0xff);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Inc,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x43);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0xff;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Inc,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x0);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0xfe;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Inc,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0xff);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_from_x_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Inx,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x43);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_from_x_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0xff;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Inx,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x0);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_from_x_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0xfe;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Inx,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0xff);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_from_y_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Iny,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x43);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_from_y_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0xff;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Iny,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x0);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_increment_one_from_y_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0xfe;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Iny,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0xff);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_shift_right_without_setting_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lsr,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x01);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_shift_right_setting_carry() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x03;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lsr,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x01);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_shift_right_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x00;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lsr,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x00);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_left_without_setting_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rol,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x04);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_left_with_carry_without_setting_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = true;
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rol,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x05);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_left_setting_carry_and_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0xC0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rol,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_left_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x00;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rol,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x00);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_right_without_setting_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ror,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x01);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_right_with_carry_without_setting_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = true;
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ror,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x81);
        assert!(!cpu.registers.p.carry);
        assert!(cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_right_setting_carry() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x03;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ror,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x01);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_rotate_right_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x00;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ror,
            addressing_mode: AddressingMode::Accumulator,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x00);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.negative);
        assert!(cpu.registers.p.zero);
    }
}