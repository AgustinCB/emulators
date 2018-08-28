use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_bcc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if !self.registers.p.carry {
            self.registers.pc += offset as u16;
        }
        Ok(())
    }

    pub(crate) fn execute_bcs(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if self.registers.p.carry {
            self.registers.pc += offset as u16;
        }
        Ok(())
    }

    pub(crate) fn execute_beq(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if self.registers.p.zero {
            self.registers.pc += offset as u16;
        }
        Ok(())
    }

    #[inline]
    fn get_branch_offset(&self, addressing_mode: &AddressingMode) -> Result<i8, CpuError> {
        match addressing_mode {
            AddressingMode::Relative { byte } => Ok(*byte),
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {AVAILABLE_MEMORY, Mos6502Cpu};

    #[test]
    fn it_should_branch_if_carry_is_clear_on_bcc() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_carry_is_set_on_bcc() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_shouldnt_branch_if_carry_is_clear_on_bcs() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_carry_is_set_on_bcs() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.carry = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_zero_is_clear_on_beq() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.zero = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Beq,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_zero_is_set_on_beq() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.zero = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Beq,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }
}