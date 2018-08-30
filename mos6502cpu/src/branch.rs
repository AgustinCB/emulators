use {Mos6502Cpu, CpuError, CpuResult};
use bit_utils::{two_bytes_to_word, word_to_two_bytes};
use instruction::AddressingMode;
use mos6502cpu::ProcessorStatus;

impl Mos6502Cpu {
    pub(crate) fn execute_bcc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if !self.registers.p.carry {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_bcs(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if self.registers.p.carry {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_beq(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if self.registers.p.zero {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_bmi(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if self.registers.p.negative {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_bne(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if !self.registers.p.zero {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_bpl(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if !self.registers.p.negative {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }
    pub(crate) fn execute_bvc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if !self.registers.p.overflow {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_bvs(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let offset = self.get_branch_offset(&addressing_mode)?;
        if self.registers.p.overflow {
            self.update_pc_from_offset(offset);
        }
        Ok(())
    }

    pub(crate) fn execute_jmp(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let address = self.get_address_from_addressing_mode(addressing_mode)?;
        self.registers.pc = address;
        Ok(())
    }

    pub(crate) fn execute_jsr(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Absolute { high_byte, low_byte } = addressing_mode {
            let address = two_bytes_to_word(*high_byte, *low_byte);
            let (low_byte, high_byte) = word_to_two_bytes(self.registers.pc);
            self.push(high_byte);
            self.push(low_byte);
            self.registers.pc = address;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_rti(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.registers.p = ProcessorStatus::from_byte(self.pull());
            let (low_byte, high_byte) = (self.pull(), self.pull());
            self.registers.pc = two_bytes_to_word(high_byte, low_byte);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_rts(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let (low_byte, high_byte) = (self.pull(), self.pull());
            self.registers.pc = two_bytes_to_word(high_byte, low_byte);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn get_branch_offset(&self, addressing_mode: &AddressingMode) -> Result<i8, CpuError> {
        match addressing_mode {
            AddressingMode::Relative { byte } => Ok(*byte),
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    #[inline]
    fn update_pc_from_offset(&mut self, offset: i8) {
        let pc = self.registers.pc;
        let new_pc = pc + offset as u16;
        self.update_page_crossed_status(pc, new_pc);
        self.registers.pc = new_pc;
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

    #[test]
    fn it_shouldnt_branch_if_negative_is_clear_on_bmi() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.negative = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bmi,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_negative_is_set_on_bmi() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.negative = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bmi,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_should_branch_if_zero_is_clear_on_bne() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.zero = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bne,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_zero_is_set_on_bne() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.zero = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bne,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_negative_is_clear_on_bpl() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.negative = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bpl,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_negative_is_set_on_bpl() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.negative = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bpl,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_overflow_is_clear_on_bvc() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.overflow = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_overflow_is_set_on_bvc() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.overflow = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_shouldnt_branch_if_overflow_is_clear_on_bvs() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.overflow = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_overflow_is_set_on_bvs() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.p.overflow = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_should_jump() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Jmp,
            addressing_mode: AddressingMode::Absolute {
                high_byte: 0x42,
                low_byte: 0x24,
            },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_jump_and_push_pc_to_stack() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.pc = 0x0042;
        cpu.registers.s = 0xff;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Jsr,
            addressing_mode: AddressingMode::Absolute {
                high_byte: 0x42,
                low_byte: 0x24,
            },
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x4224);
        assert_eq!(cpu.memory[0xff], 0x00);
        assert_eq!(cpu.memory[0xfe], 0x42);
    }

    #[test]
    fn it_should_return_from_interrupt() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.pc = 0x0042;
        cpu.registers.s = 0xfc;
        cpu.registers.p.carry = false;
        cpu.memory[0xfd] = 0x01;
        cpu.memory[0xfe] = 0x00;
        cpu.memory[0xff] = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rti,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x4200);
        assert_eq!(cpu.registers.s, 0xff);
        assert!(cpu.registers.p.carry);
    }

    #[test]
    fn it_should_return_from_subroutine() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.pc = 0x0042;
        cpu.registers.s = 0xfd;
        cpu.memory[0xfe] = 0x00;
        cpu.memory[0xff] = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rts,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.pc, 0x4200);
        assert_eq!(cpu.registers.s, 0xff);
    }
}