use bit_utils::{two_bytes_to_word, two_complement, word_to_two_bytes};
use instruction::AddressingMode;
use mos6502cpu::{ProcessorStatus, INTERRUPT_HANDLERS_START};
use {CpuError, CpuResult, Mos6502Cpu};

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

    pub(crate) fn execute_brk(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            if !self.registers.p.interrupt_disable {
                self.registers.p.break_flag = true;
                self.execute_interruption(2);
            }
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
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
        self.check_jmp_address(addressing_mode)?;
        let address = self.get_address_from_addressing_mode(addressing_mode)?;
        self.registers.pc = address;
        Ok(())
    }

    pub(crate) fn execute_jsr(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Absolute {
            high_byte,
            low_byte,
        } = addressing_mode
        {
            let address = two_bytes_to_word(*high_byte, *low_byte);
            let (low_byte, high_byte) = word_to_two_bytes(self.registers.pc - 1);
            self.push(high_byte);
            self.push(low_byte);
            self.registers.pc = address;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_nmi(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.execute_interruption(0);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_rst(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            if !self.registers.p.interrupt_disable {
                self.execute_interruption(1);
            }
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
            self.registers.pc = two_bytes_to_word(high_byte, low_byte) + 1;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn execute_interruption(&mut self, index: u16) {
        let p_byte = self.registers.p.to_byte();
        let (low_byte, high_byte) = word_to_two_bytes(self.registers.pc + 1);
        self.push(high_byte);
        self.push(low_byte);
        self.push(p_byte);
        let high_byte = self
            .memory
            .get(INTERRUPT_HANDLERS_START as u16 + index * 2 + 1);
        let low_byte = self.memory.get(INTERRUPT_HANDLERS_START as u16 + index * 2);
        let handler = two_bytes_to_word(high_byte, low_byte);
        self.registers.pc = handler;
        self.registers.p.interrupt_disable = true;
    }

    #[inline]
    fn get_branch_offset(&self, addressing_mode: &AddressingMode) -> Result<u8, CpuError> {
        match addressing_mode {
            AddressingMode::Relative { byte } => Ok(*byte),
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    #[inline]
    fn update_pc_from_offset(&mut self, offset: u8) {
        let pc = self.registers.pc;
        let new_pc = if offset & 0x80 == 0 {
            pc + offset as u16
        } else {
            pc - two_complement(offset) as u16
        };
        self.update_page_crossed_status(pc, new_pc);
        self.registers.pc = new_pc;
    }

    #[inline]
    fn check_jmp_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Indirect {
                low_byte: _,
                high_byte: _,
            } => Ok(()),
            AddressingMode::Absolute {
                low_byte: _,
                high_byte: _,
            } => Ok(()),
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
    fn it_should_branch_if_carry_is_clear_on_bcc() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.carry = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_should_branch_back_if_carry_is_clear_on_bcc_and_argument_is_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.carry = false;
        cpu.registers.pc = 1;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcc,
            addressing_mode: AddressingMode::Relative { byte: 0xff },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_shouldnt_branch_if_carry_is_set_on_bcc() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.carry = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_shouldnt_branch_if_carry_is_clear_on_bcs() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.carry = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_carry_is_set_on_bcs() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.carry = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bcs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_zero_is_clear_on_beq() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.zero = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Beq,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_zero_is_set_on_beq() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.zero = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Beq,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_negative_is_clear_on_bmi() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.negative = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bmi,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_negative_is_set_on_bmi() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.negative = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bmi,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_should_branch_if_zero_is_clear_on_bne() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.zero = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bne,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_zero_is_set_on_bne() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.zero = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bne,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_negative_is_clear_on_bpl() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.negative = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bpl,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_negative_is_set_on_bpl() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.negative = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bpl,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_change_program_counter_to_fffe_on_break() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0xfffe, 0x24);
        cpu.memory.set(0xffff, 0x42);
        cpu.registers.p.break_flag = false;
        cpu.registers.pc = 2;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Brk,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert!(cpu.registers.p.break_flag);
        assert!(cpu.registers.p.interrupt_disable);
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_save_in_stack_status_on_break() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.s = 3;
        cpu.registers.pc = 0x4224;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Brk,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert_eq!(cpu.registers.s, 0);
        assert_eq!(cpu.memory.get(0x103), 0x42);
        assert_eq!(cpu.memory.get(0x102), 0x25);
        assert_eq!(cpu.memory.get(0x101), 0x30);
    }

    #[test]
    fn it_should_branch_if_overflow_is_clear_on_bvc() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.overflow = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_shouldnt_branch_if_overflow_is_set_on_bvc() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.overflow = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvc,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_shouldnt_branch_if_overflow_is_clear_on_bvs() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.overflow = false;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x0);
    }

    #[test]
    fn it_should_branch_if_overflow_is_set_on_bvs() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.overflow = true;
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Bvs,
            addressing_mode: AddressingMode::Relative { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x42);
    }

    #[test]
    fn it_should_change_program_counter_to_fffe_on_irq() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0xfffe, 0x24);
        cpu.memory.set(0xffff, 0x42);
        cpu.registers.p.break_flag = false;
        cpu.registers.pc = 2;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Irq,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert!(cpu.registers.p.break_flag);
        assert!(cpu.registers.p.interrupt_disable);
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_save_in_stack_status_on_irq() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.s = 3;
        cpu.registers.pc = 0x4224;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Irq,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert_eq!(cpu.registers.s, 0);
        assert_eq!(cpu.memory.get(0x103), 0x42);
        assert_eq!(cpu.memory.get(0x102), 0x25);
        assert_eq!(cpu.memory.get(0x101), 0x30);
    }

    #[test]
    fn it_should_jump() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.pc = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Jmp,
            addressing_mode: AddressingMode::Absolute {
                high_byte: 0x42,
                low_byte: 0x24,
            },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_jump_and_push_pc_to_stack() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.pc = 0x0042;
        cpu.registers.s = 0xff;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Jsr,
            addressing_mode: AddressingMode::Absolute {
                high_byte: 0x42,
                low_byte: 0x24,
            },
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x4224);
        assert_eq!(cpu.memory.get(0x1ff), 0x00);
        assert_eq!(cpu.memory.get(0x1fe), 0x41);
    }

    #[test]
    fn it_should_change_program_counter_to_fffe_on_nmi() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0xfffa, 0x24);
        cpu.memory.set(0xfffb, 0x42);
        cpu.registers.p.break_flag = false;
        cpu.registers.pc = 2;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Nmi,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert!(!cpu.registers.p.break_flag);
        assert!(cpu.registers.p.interrupt_disable);
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_save_in_stack_status_on_nmi() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.s = 3;
        cpu.registers.pc = 0x4224;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Nmi,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert_eq!(cpu.registers.s, 0);
        assert_eq!(cpu.memory.get(0x103), 0x42);
        assert_eq!(cpu.memory.get(0x102), 0x25);
        assert_eq!(cpu.memory.get(0x101), 0x30);
    }

    #[test]
    fn it_should_ignore_interrupt_disable_on_nmi() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.p.break_flag = false;
        cpu.registers.p.interrupt_disable = true;
        cpu.memory.set(0xfffa, 0x24);
        cpu.memory.set(0xfffb, 0x42);
        cpu.registers.pc = 2;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Nmi,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert!(!cpu.registers.p.break_flag);
        assert!(cpu.registers.p.interrupt_disable);
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_change_program_counter_to_fffe_on_rst() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0xfffc, 0x24);
        cpu.memory.set(0xfffd, 0x42);
        cpu.registers.p.break_flag = false;
        cpu.registers.pc = 2;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rst,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert!(!cpu.registers.p.break_flag);
        assert!(cpu.registers.p.interrupt_disable);
        assert_eq!(cpu.registers.pc, 0x4224);
    }

    #[test]
    fn it_should_save_in_stack_status_on_rst() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.s = 3;
        cpu.registers.pc = 0x4224;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rst,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert_eq!(cpu.registers.s, 0);
        assert_eq!(cpu.memory.get(0x103), 0x42);
        assert_eq!(cpu.memory.get(0x102), 0x25);
        assert_eq!(cpu.memory.get(0x101), 0x30);
    }

    #[test]
    fn it_should_return_from_interrupt() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.pc = 0x0042;
        cpu.registers.s = 0xfc;
        cpu.registers.p.carry = false;
        cpu.memory.set(0x1fd, 0x01);
        cpu.memory.set(0x1fe, 0x00);
        cpu.memory.set(0x1ff, 0x42);
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rti,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x4200);
        assert_eq!(cpu.registers.s, 0xff);
        assert!(cpu.registers.p.carry);
    }

    #[test]
    fn it_should_return_from_subroutine() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.pc = 0x0042;
        cpu.registers.s = 0xfd;
        cpu.memory.set(0x1fe, 0xff);
        cpu.memory.set(0x1ff, 0x41);
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Rts,
            addressing_mode: AddressingMode::Implicit,
        })
        .unwrap();
        assert_eq!(cpu.registers.pc, 0x4200);
        assert_eq!(cpu.registers.s, 0xff);
    }
}
