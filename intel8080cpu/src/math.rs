use super::CpuError;
use intel8080cpu::{Intel8080Cpu, RegisterType};

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn execute_aci(&mut self, byte: u8) -> Result<(), CpuError> {
        let carry_as_u16 = self.flags.carry as u16;
        let destiny_value = (self.get_current_a_value()? as u16 + carry_as_u16) & 0xff;
        let new_value = self.perform_add_with_carry(byte as u16, destiny_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_adi(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let new_value = self.perform_add_with_carry(byte as u16, destiny_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_adc_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_current_single_register_value(register_type)? as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_adc_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_add_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_current_single_register_value(register_type)? as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_add_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        self.save_to_a(new_value)
    }

    #[inline]
    pub(crate) fn execute_cmp_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_current_single_register_value(register_type)? as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
        Ok(())
    }

    #[inline]
    pub(crate) fn execute_cmp_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
        Ok(())
    }

    #[inline]
    pub(crate) fn execute_cpi(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        self.perform_sub_with_carry(destiny_value, byte as u16);
        Ok(())
    }

    pub(crate) fn execute_daa(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let mut least_significant = destiny_value & 0x0f;
        let mut result = destiny_value;
        if least_significant > 9 || self.flags.auxiliary_carry {
            result += 6;
            self.flags.auxiliary_carry = (least_significant + 6) > 0x0f;
            least_significant = result & 0x0f;
        }
        let mut most_significant = (result & 0xf0) >> 4;
        if most_significant > 9 || self.flags.carry {
            most_significant += 6;
            self.flags.carry = most_significant > 0x0f;
            most_significant &= 0x0f;
        }
        result = (most_significant << 4) | least_significant;
        self.update_flags(result, false);
        self.save_to_a(result as u8)
    }

    pub(crate) fn execute_dad(&mut self, register_type: &RegisterType) -> Result<(), CpuError> {
        let destiny_value = self.get_current_hl_value() as u32;
        let source_value = match register_type {
            RegisterType::B => Ok(self.get_current_bc_value() as u32),
            RegisterType::D => Ok(self.get_current_de_value() as u32),
            RegisterType::H => Ok(self.get_current_hl_value() as u32),
            RegisterType::Sp => Ok(self.get_current_sp_value() as u32),
            _ => Err(CpuError::InvalidRegisterArgument {
                register: *register_type,
            }),
        }?;
        let result = destiny_value + source_value;
        self.flags.carry = result > 0xffff;
        self.save_to_single_register((result >> 8) as u8, &RegisterType::H)?;
        self.save_to_single_register(result as u8, &RegisterType::L)
    }

    pub(crate) fn execute_dcr_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let source_value = self.get_current_single_register_value(register_type)? as u16;
        let new_value = self.perform_sub_without_carry(source_value, 1);
        self.save_to_single_register(new_value, register_type)
    }

    pub(crate) fn execute_dcr_by_memory(&mut self) {
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_sub_without_carry(source_value, 1);
        self.set_value_in_memory_at_hl(new_value);
    }

    pub(crate) fn execute_dcx(&mut self, register_type: &RegisterType) -> Result<(), CpuError> {
        self.perform_step_on_double_register(register_type, false)
    }

    pub(crate) fn execute_inr_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let source_value = self.get_current_single_register_value(register_type)? as u16;
        let new_value = self.perform_add_without_carry(source_value, 1);
        self.save_to_single_register(new_value, register_type)
    }

    pub(crate) fn execute_inr_by_memory(&mut self) {
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_add_without_carry(source_value, 1);
        self.set_value_in_memory_at_hl(new_value);
    }

    pub(crate) fn execute_inx(&mut self, register_type: &RegisterType) -> Result<(), CpuError> {
        self.perform_step_on_double_register(register_type, true)
    }

    pub(crate) fn execute_sbb_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let carry = self.flags.carry as u8;
        let source_value =
            ((self.get_current_single_register_value(register_type)? + carry) & 0xff) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_sbb_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let carry = self.flags.carry as u8;
        let source_value = (self.get_value_in_memory_at_hl() + carry) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_sbi(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let add = byte as u16 + self.flags.carry as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, add);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_sub_by_register(
        &mut self,
        register_type: &RegisterType,
    ) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_current_single_register_value(register_type)? as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_sub_by_memory(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value)
    }

    pub(crate) fn execute_sui(&mut self, byte: u8) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()? as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, byte as u16);
        self.save_to_a(new_value)
    }

    #[inline]
    fn perform_add_with_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_add(destiny, source, true)
    }

    #[inline]
    fn perform_add_without_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_add(destiny, source, false)
    }

    #[inline]
    fn perform_step_on_double_register(
        &mut self,
        register_type: &RegisterType,
        inc: bool,
    ) -> Result<(), CpuError> {
        let destiny_value = match register_type {
            RegisterType::B => self.get_current_bc_value() as u32,
            RegisterType::D => self.get_current_de_value() as u32,
            RegisterType::H => self.get_current_hl_value() as u32,
            RegisterType::Sp => self.get_current_sp_value() as u32,
            _ => panic!("{} is not a valid INX argument!", register_type.to_string()),
        };
        let result = if inc {
            destiny_value.wrapping_add(1)
        } else {
            destiny_value.wrapping_sub(1)
        };
        match register_type {
            RegisterType::B => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::B)?;
                self.save_to_single_register(result as u8, &RegisterType::C)
            }
            RegisterType::D => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::D)?;
                self.save_to_single_register(result as u8, &RegisterType::E)
            }
            RegisterType::H => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::H)?;
                self.save_to_single_register(result as u8, &RegisterType::L)
            }
            RegisterType::Sp => Ok(self.save_to_sp(result as u16)),
            _ => Err(CpuError::InvalidRegisterArgument {
                register: *register_type,
            }),
        }
    }

    #[inline]
    fn perform_add(&mut self, destiny: u16, source: u16, with_carry: bool) -> u8 {
        let answer: u16 = source + destiny;
        self.update_flags(answer, with_carry);
        self.update_auxiliary_carry(destiny, source);
        (answer & 0xff) as u8
    }

    #[inline]
    fn perform_sub_with_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_sub(destiny, source, true)
    }

    #[inline]
    fn perform_sub_without_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_sub(destiny, source, false)
    }

    #[inline]
    fn perform_sub(&mut self, destiny: u16, source: u16, with_carry: bool) -> u8 {
        let answer = destiny + (!source & 0xff) + 1;
        self.update_flags(answer, false);
        if with_carry {
            self.flags.carry = answer <= 0xff;
        }
        self.update_auxiliary_carry_with_sub(destiny, source);
        (answer & 0xff) as u8
    }

    #[inline]
    fn update_auxiliary_carry_with_sub(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (!source & 0x0f) + 1 > 0x0f;
    }

    #[inline]
    fn update_auxiliary_carry(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (source & 0x0f) > 0x0f;
    }
}

#[cfg(test)]
mod tests {
    use super::super::cpu::Cpu;
    use instruction::Intel8080Instruction;
    use intel8080cpu::{Intel8080Cpu, Location, RegisterType, ROM_MEMORY_LIMIT};

    #[test]
    fn it_should_execute_aci_without_carry() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.save_to_a(0x56).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Aci { byte: 0xbe })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x14);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_aci_with_carry() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = true;
        cpu.save_to_a(0x14).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Aci { byte: 0x42 })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x57);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_adi() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x56).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Adi { byte: 0xbe })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x14);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_adc_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42).unwrap();
        cpu.save_to_single_register(0x3d, &RegisterType::C).unwrap();
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Adc {
            source: Location::Register {
                register: RegisterType::C,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x7f);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_adc_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::L).unwrap();
        cpu.memory[0] = 0x3d;
        cpu.flags.carry = true;
        cpu.execute_instruction(&Intel8080Instruction::Adc {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x80);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_add_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x21).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Add {
            source: Location::Register {
                register: RegisterType::A,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x42);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_add_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x6c).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::L).unwrap();
        cpu.memory[0] = 0x2e;
        cpu.execute_instruction(&Intel8080Instruction::Add {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x9a);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_cmp_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x0a).unwrap();
        cpu.save_to_single_register(0x05, &RegisterType::E).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Cmp {
            source: Location::Register {
                register: RegisterType::E,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x0a);
        assert_eq!(
            cpu.get_current_single_register_value(&RegisterType::E)
                .unwrap(),
            0x05
        );
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_cmp_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x02).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::L).unwrap();
        cpu.memory[0] = 0x05;
        cpu.execute_instruction(&Intel8080Instruction::Cmp {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x02);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_cpi() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x4a).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Cpi { byte: 0x40 })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x4a);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_daa_without_carries_nor_change() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x55).unwrap();
        cpu.flags.auxiliary_carry = false;
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Daa).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x55);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_daa_with_carries() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x10).unwrap();
        cpu.flags.auxiliary_carry = true;
        cpu.flags.carry = true;
        cpu.execute_instruction(&Intel8080Instruction::Daa).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x76);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_daa_without_carries_but_with_change_without_carry() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xaa).unwrap();
        cpu.flags.auxiliary_carry = false;
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Daa).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x10);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dad() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x33, &RegisterType::B).unwrap();
        cpu.save_to_single_register(0x9f, &RegisterType::C).unwrap();
        cpu.save_to_single_register(0xa1, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x7b, &RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Dad {
            register: RegisterType::B,
        })
        .unwrap();
        assert_eq!(cpu.get_current_hl_value(), 0xd51a);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcr_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x40).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Dcr {
            source: Location::Register {
                register: RegisterType::A,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x3f);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcr_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x3a, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x7c, &RegisterType::L).unwrap();
        cpu.memory[0x3a7c] = 0x40;
        cpu.execute_instruction(&Intel8080Instruction::Dcr {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.memory[0x3a7c], 0x3f);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcx() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x98, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x00, &RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Dcx {
            register: RegisterType::H,
        })
        .unwrap();
        assert_eq!(cpu.get_current_hl_value(), 0x97ff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcx_when_zero() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x00, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x00, &RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Dcx {
            register: RegisterType::H,
        })
        .unwrap();
        assert_eq!(cpu.get_current_hl_value(), 0xffff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_inr_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x99, &RegisterType::C).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Inr {
            source: Location::Register {
                register: RegisterType::C,
            },
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(&RegisterType::C)
                .unwrap(),
            0x9a
        );
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_inr_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x3a, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x7c, &RegisterType::L).unwrap();
        cpu.memory[0x3a7c] = 0x99;
        cpu.execute_instruction(&Intel8080Instruction::Inr {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.memory[0x3a7c], 0x9a);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_inx() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x38, &RegisterType::D).unwrap();
        cpu.save_to_single_register(0xff, &RegisterType::E).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Inx {
            register: RegisterType::D,
        })
        .unwrap();
        assert_eq!(cpu.get_current_de_value(), 0x3900);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbb_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x04).unwrap();
        cpu.save_to_single_register(0x02, &RegisterType::L).unwrap();
        cpu.flags.carry = true;
        cpu.execute_instruction(&Intel8080Instruction::Sbb {
            source: Location::Register {
                register: RegisterType::L,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x01);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbb_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x04).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::L).unwrap();
        cpu.memory[0] = 0x02;
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Sbb {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x02);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbi_without_carry() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0).unwrap();
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Sbi { byte: 0x01 })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbi_with_carry() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0).unwrap();
        cpu.flags.carry = true;
        cpu.execute_instruction(&Intel8080Instruction::Sbi { byte: 0x01 })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xfe);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sub_by_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3e).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Sub {
            source: Location::Register {
                register: RegisterType::A,
            },
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sub_by_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3e).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::H).unwrap();
        cpu.save_to_single_register(0x0, &RegisterType::L).unwrap();
        cpu.memory[0] = 0x3d;
        cpu.execute_instruction(&Intel8080Instruction::Sub {
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x01);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sui() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0).unwrap();
        cpu.flags.carry = false;
        cpu.execute_instruction(&Intel8080Instruction::Sui { byte: 0x01 })
            .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0xff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }
}
