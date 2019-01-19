use instruction::AddressingMode;
use {CpuResult, Mos6502Cpu};

impl Mos6502Cpu {
    pub(crate) fn execute_and(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(&addressing_mode)?;
        self.execute_and_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_and_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = self.registers.a & value;
        self.registers.a = answer;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        Ok(())
    }

    pub(crate) fn execute_eor(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(&addressing_mode)?;
        self.execute_eor_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_eor_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = self.registers.a ^ value;
        self.registers.a = answer;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        Ok(())
    }

    pub(crate) fn execute_ora(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(&addressing_mode)?;
        self.execute_ora_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_ora_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        let answer = self.registers.a | value;
        self.registers.a = answer;
        self.update_zero_flag(answer);
        self.update_negative_flag(answer);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {Mos6502Cpu, AVAILABLE_MEMORY};

    #[test]
    fn it_should_execute_and_not_set_anything() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x03;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::And,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x01);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_and_set_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x03;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::And,
            addressing_mode: AddressingMode::Immediate { byte: 0x04 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_and_set_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::And,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_perform_exclusive_or() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Eor,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x03);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_perform_exclusive_or_and_set_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x02;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Eor,
            addressing_mode: AddressingMode::Immediate { byte: 0x02 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_perform_exclusive_or_and_set_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Eor,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_inclusive_or_and_set_nothing() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ora,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_inclusive_or_and_set_zero() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ora,
            addressing_mode: AddressingMode::Immediate { byte: 0x0 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_inclusive_or_and_set_negative() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ora,
            addressing_mode: AddressingMode::Immediate { byte: 0x0 },
        })
        .unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }
}
