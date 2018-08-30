use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_lda(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        self.registers.a = value;
        self.update_zero_flag(value);
        self.update_negative_flag(value);
        Ok(())
    }

    pub(crate) fn execute_ldx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_load_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        self.registers.x = value;
        self.update_zero_flag(value);
        self.update_negative_flag(value);
        Ok(())
    }

    pub(crate) fn execute_ldy(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_load_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        self.registers.y = value;
        self.update_zero_flag(value);
        self.update_negative_flag(value);
        Ok(())
    }

    pub(crate) fn execute_sta(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        let address = self.get_address_from_addressing_mode(addressing_mode)? as usize;
        self.memory[address] = self.registers.a;
        Ok(())
    }

    pub(crate) fn execute_stx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        let address = self.get_address_from_addressing_mode(addressing_mode)? as usize;
        self.memory[address] = self.registers.x;
        Ok(())
    }

    pub(crate) fn execute_sty(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        let address = self.get_address_from_addressing_mode(addressing_mode)? as usize;
        self.memory[address] = self.registers.y;
        Ok(())
    }

    #[inline]
    fn check_data_load_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn check_data_store_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::IndexedIndirect { byte: _ } => Ok(()),
            AddressingMode::IndirectIndexed { byte: _ } => Ok(()),
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
    fn it_should_load_into_accumulator_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lda,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_accumulator_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lda,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_accumulator_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lda,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_x_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldx,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_x_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldx,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_x_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldx,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_y_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldy,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_y_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldy,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_y_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldy,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_store_the_accumulator() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0;
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sta,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
    }

    #[test]
    fn it_should_store_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0;
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Stx,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
    }

    #[test]
    fn it_should_store_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0;
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sty,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
    }
}