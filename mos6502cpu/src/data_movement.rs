use {Mos6502Cpu, CpuResult, CpuError};
use instruction::AddressingMode;

const ONE_TWO_COMPLEMENT: u8 = 0xff;

impl Mos6502Cpu {
    pub(crate) fn execute_asl(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
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

    pub(crate) fn execute_dec(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_memory_data_movement_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        let answer = self.decrement(value);
        self.set_value_to_addressing_mode(addressing_mode, answer)?;
        Ok(())
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

    #[inline]
    fn decrement(&mut self, value: u8) -> u8 {
        let answer = (value as u16 + ONE_TWO_COMPLEMENT as u16) as u8;
        self.registers.p.zero = answer == 0;
        self.registers.p.negative = answer & 0x80 > 0;
        answer
    }

    #[inline]
    fn check_data_movement_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
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
    fn check_memory_data_movement_address(
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
}