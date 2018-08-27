use {Mos6502Cpu, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_and(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(&addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode);
        let answer = self.registers.a & value;
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
    use {AVAILABLE_MEMORY, Mos6502Cpu};

    #[test]
    fn it_should_execute_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x03;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::And,
            addressing_mode: AddressingMode::Immediate { byte: 0x01 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x01);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x03;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::And,
            addressing_mode: AddressingMode::Immediate { byte: 0x04 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_execute_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x80;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::And,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }
}