use super::CpuError;
use intel8080cpu::{Intel8080Cpu, RegisterType};

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn execute_ana_by_register(
        &mut self,
        register_type: RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let source_value = self.get_current_single_register_value(register_type)?;
        let new_value = self.perform_and(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_ana_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_and(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_ani(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let new_value = self.perform_and(destiny_value, byte);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_ora_by_register(
        &mut self,
        register_type: RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let source_value = self.get_current_single_register_value(register_type)?;
        let new_value = self.perform_or(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_ora_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_or(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_ori(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let new_value = self.perform_or(destiny_value, byte);
        self.save_to_a(new_value)
    }

    #[inline]
    pub(crate) fn execute_ral(&mut self) -> Result<(), CpuError> {
        let a_value = self.get_current_a_value()?;
        let operand = if self.flags.carry {
            a_value | 0x80
        } else {
            a_value & (!0x80)
        };
        self.flags.carry = (a_value & 0x80) == 0x80;
        self.save_to_a(operand.rotate_left(1))
    }

    #[inline]
    pub(crate) fn execute_rar(&mut self) -> Result<(), CpuError> {
        let a_value = self.get_current_a_value()?;
        let new_a_value = if self.flags.carry {
            a_value.rotate_right(1) | 0x80
        } else {
            a_value.rotate_right(1) & (!0x80)
        };
        self.save_to_a(new_a_value)?;
        self.flags.carry = (a_value & 0x01) == 0x01;
        Ok(())
    }

    #[inline]
    pub(crate) fn execute_rlc(&mut self) -> Result<(), CpuError> {
        let value = self.get_current_a_value()?.rotate_left(1);
        self.flags.carry = (value & 0x01) != 0;
        self.save_to_a(value)
    }

    #[inline]
    pub(crate) fn execute_rrc(&mut self) -> Result<(), CpuError> {
        let value = self.get_current_a_value()?.rotate_right(1);
        self.flags.carry = (value & 0x80) != 0;
        self.save_to_a(value)
    }

    pub(crate) fn execute_xra_by_register(
        &mut self,
        register_type: RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let source_value = self.get_current_single_register_value(register_type)?;
        let new_value = self.perform_xor(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_xra_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_xor(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_xri(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        let new_value = self.perform_xor(destiny_value, byte);
        self.save_to_a(new_value)?;
        Ok(())
    }

    #[inline]
    fn perform_and(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny & source;
        self.update_flags(u16::from(answer), false);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        answer
    }

    #[inline]
    fn perform_or(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny | source;
        self.update_flags(u16::from(answer), false);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        answer
    }

    #[inline]
    fn perform_xor(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny ^ source;
        self.update_flags(u16::from(answer), false);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        answer
    }
}

#[cfg(test)]
mod tests {
    use super::super::cpu::Cpu;
    use instruction::Intel8080Instruction;
    use intel8080cpu::{Intel8080Cpu, Location, RegisterType, ROM_MEMORY_LIMIT};

    #[test]
    fn it_should_execute_ana_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xfc).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::L).unwrap();
        cpu.memory[0] = 0x0f;
        cpu.execute_instruction(&Intel8080Instruction::Ana {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x0c);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ana_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xfc).unwrap();
        cpu.save_to_single_register(0x0f, RegisterType::C).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Ana {
            source: Location::Register {
                register: RegisterType::C,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x0c);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ani() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3a).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Ani { byte: 0x0f })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x0a);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ora_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x33).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::L).unwrap();
        cpu.memory[0] = 0x0f;
        cpu.execute_instruction(&Intel8080Instruction::Ora {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x3f);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ora_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x33).unwrap();
        cpu.save_to_single_register(0x0f, RegisterType::C).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Ora {
            source: Location::Register {
                register: RegisterType::C,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x3f);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ori() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xb5).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Ori { byte: 0x0f })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xbf);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ral() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xb5).unwrap();
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Ral).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x6a);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_rar() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x6a).unwrap();
        cpu.flags.carry = true;
        cpu.execute_instruction(&Intel8080Instruction::Rar).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xb5);
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_rlc() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xf2).unwrap();
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Rlc).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xe5);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_rrc() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xf2).unwrap();
        cpu.flags.carry = true;
        cpu.execute_instruction(&Intel8080Instruction::Rrc).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x79);
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_xri() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3b).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Xri { byte: 0x81 })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xba);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_xra_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x78).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::L).unwrap();
        cpu.memory[0] = 0x5c;
        cpu.execute_instruction(&Intel8080Instruction::Xra {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x24);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_xra_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xff).unwrap();
        cpu.save_to_single_register(0x0f, RegisterType::C).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Xra {
            source: Location::Register {
                register: RegisterType::C,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xf0);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_xra_on_itself() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x33).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Xra {
            source: Location::Register {
                register: RegisterType::A,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.zero);
    }
}
