use {Mos6502Cpu, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_adc(&mut self, addressing_mode: AddressingMode) -> CpuResult {
        self.check_alu_address(&addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode) as u16;
        let carry_as_u16 = self.registers.p.carry as u16;
        let new_a = self.registers.a as u16 + value + carry_as_u16;
        self.registers.p.zero = (new_a as u8) == 0;
        self.registers.p.carry = new_a > 0xff;
        self.registers.p.negative = new_a & 0x80 > 0;
        self.registers.p.overflow =
            self.calculate_overflow(self.registers.a, (value + carry_as_u16) as u8, new_a as u8);
        self.registers.a = new_a as u8;
        Ok(())
    }

    #[inline]
    fn calculate_overflow(&self, op1: u8, op2: u8, result: u8) -> bool {
        ((op1 & 0x80) > 0 && (op2 & 0x80) > 0 && (result & 0x80) == 0) ||
            ((op1 & 0x80) == 0 && (op2 & 0x80) == 0 && (result & 0x80) > 0)
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {AVAILABLE_MEMORY, Mos6502Cpu};

    #[test]
    fn it_should_adc_without_prev_carry_overflow_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x21;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x21 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_without_overflow_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x21;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x21 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x43);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_overflow_without_prev_carry_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x84);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_overflow_without_carry_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x85);
        assert!(!cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_carry_without_prev_carry_overflow_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x84;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x84 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x08);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_carry_without_overflow_negative_nor_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x84;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0x84 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x09);
        assert!(cpu.registers.p.carry);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_zero_without_prev_carry_overflow_carry_nor_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.p.carry = false;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xbe },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_adc_with_prev_carry_zero_without_overflow_carry_nor_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x41;
        cpu.registers.p.carry = true;
        cpu.execute_instruction(Mos6502Instruction {
            instruction: Mos6502InstructionCode::Adc,
            addressing_mode: AddressingMode::Immediate { byte: 0xbe },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.carry);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.overflow);
        assert!(!cpu.registers.p.negative);
    }
}