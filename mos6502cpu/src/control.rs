use {Mos6502Cpu, CpuError, CpuResult};
use bit_utils::{two_bytes_to_word, word_to_two_bytes};
use instruction::AddressingMode;
use mos6502cpu::INTERRUPT_HANDLERS_START;

impl Mos6502Cpu {
    pub(crate) fn execute_bit(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_bit_address(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        let answer = value & self.registers.a;
        self.update_zero_flag(answer);
        self.registers.p.overflow = value & 0x40 > 0;
        self.registers.p.carry = value & 0x80 > 0;
        Ok(())
    }

    pub(crate) fn execute_brk(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p.break_flag = true;
            let p_byte = self.registers.p.to_byte();
            let (low_byte, high_byte) = word_to_two_bytes(self.registers.pc);
            self.push(high_byte);
            self.push(low_byte);
            self.push(p_byte);
            let high_byte = self.memory[INTERRUPT_HANDLERS_START + 2 * 2 + 1];
            let low_byte = self.memory[INTERRUPT_HANDLERS_START + 2 * 2];
            let handler = two_bytes_to_word(high_byte, low_byte);
            self.registers.pc = handler;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
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
            self.registers.p.interrupt = false;
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
            self.registers.p.interrupt = true;
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
    fn it_should_change_program_counter_to_fffe_on_break() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0xfffe] = 0x24;
        cpu.memory[0xffff] = 0x42;
        cpu.registers.p.break_flag = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Brk,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(cpu.registers.p.break_flag);
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_save_in_stack_status_on_break() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 3;
        cpu.registers.pc = 0x4224;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Brk,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.s, 0);
        assert_eq!(cpu.memory[3], 0x42);
        assert_eq!(cpu.memory[2], 0x24);
        assert_eq!(cpu.memory[1], 0x30);
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
        cpu.registers.p.interrupt = true;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Cli,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(!cpu.registers.p.interrupt);
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
        cpu.registers.p.interrupt = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sei,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert!(cpu.registers.p.interrupt);
    }
}