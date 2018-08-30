use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;
use mos6502cpu::ProcessorStatus;

impl Mos6502Cpu {
    pub(crate) fn execute_pha(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let a = self.registers.a;
            self.push(a);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_php(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let status = self.registers.p.to_byte();
            self.push(status);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_pla(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let new_a = self.pull();
            self.registers.a = new_a;
            self.update_negative_flag(new_a);
            self.update_zero_flag(new_a);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_plp(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let new_status = self.pull();
            self.registers.p = ProcessorStatus::from_byte(new_status);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {AVAILABLE_MEMORY, Mos6502Cpu};

    #[test]
    fn it_should_push_accumulator_onto_stack() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0xff;
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Pha,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.memory[0xff], 0x42);
        assert_eq!(cpu.registers.s, 0xfe);
    }

    #[test]
    fn it_should_push_status_onto_stack() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0xff;
        cpu.registers.p.carry = true;
        let expected = cpu.registers.p.to_byte();
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Php,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.memory[0xff], expected);
        assert_eq!(cpu.registers.s, 0xfe);
    }

    #[test]
    fn it_should_pull_accumulator_without_setting_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0xfe;
        cpu.registers.a = 0;
        cpu.memory[0xff] = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Pla,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert_eq!(cpu.registers.s, 0xff);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_pull_accumulator_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0xfe;
        cpu.registers.a = 0x42;
        cpu.memory[0xff] = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Pla,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert_eq!(cpu.registers.s, 0xff);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_pull_accumulator_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0xfe;
        cpu.registers.a = 0x42;
        cpu.memory[0xff] = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Pla,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert_eq!(cpu.registers.s, 0xff);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_pull_status() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0xfe;
        cpu.registers.p.carry = true;
        cpu.memory[0xff] = cpu.registers.p.to_byte();
        cpu.registers.p.carry = false;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Plp,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.s, 0xff);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.interrupt_disable);
        assert!(!cpu.registers.p.negative);
        assert!(!cpu.registers.p.decimal);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.break_flag);
    }
}