use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_bit(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_bit_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = value & self.registers.a;
        self.update_zero_flag(answer);
        self.registers.p.overflow = value & 0x40 > 0;
        self.registers.p.carry = value & 0x80 > 0;
        Ok(())
    }

    pub(crate) fn execute_clc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.carry = false;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_cld(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.decimal = false;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_cli(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.interrupt_disable = false;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_clv(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.overflow = false;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_sec(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.carry = true;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_sed(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.decimal = true;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_sei(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.interrupt_disable = true;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn check_bit_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
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
    fn it_shouldnt_set_zero_when_bit_on_same_value() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x42;
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bit,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
        assert!(!cpu.registers.p.zero);
    }

    #[test]
    fn it_should_set_zero_when_bit_on_different_value() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x42;
        cpu.registers.a = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bit,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
        assert!(cpu.registers.p.zero);
    }

    #[test]
    fn it_should_set_overflow_and_carry_from_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0x42;
        cpu.registers.a = 0x0;
        cpu.registers.p.overflow = false;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bit,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.carry);
    }

    #[test]
    fn it_should_set_carry_to_zero_on_clc() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Clc,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(!cpu.registers.p.carry);
    }

    #[test]
    fn it_should_set_decimal_to_zero_on_cld() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.decimal = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cld,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(!cpu.registers.p.decimal);
    }

    #[test]
    fn it_should_set_interrupt_to_zero_on_cli() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.interrupt_disable = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cli,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(!cpu.registers.p.interrupt_disable);
    }

    #[test]
    fn it_should_set_overflow_to_zero_on_clv() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.overflow = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Clv,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(!cpu.registers.p.overflow);
    }

    #[test]
    fn it_should_set_carry_on_sec() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sec,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(cpu.registers.p.carry);
    }

    #[test]
    fn it_should_set_decimal_on_sed() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.decimal = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sed,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(cpu.registers.p.decimal);
    }

    #[test]
    fn it_should_set_interrupt_on_sei() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.interrupt_disable = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sei,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(cpu.registers.p.interrupt_disable);
    }
}