use cpu::cpu::Cpu;
use disassembler_8080::RegisterType;

impl Cpu {
    pub(crate) fn execute_aci(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(byte as u16, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_adi(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        let new_value = self.perform_add_with_carry(byte as u16, destiny_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_adc_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_adc_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_add_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_add_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    #[inline]
    pub(crate) fn execute_cmp_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
    }

    #[inline]
    pub(crate) fn execute_cpi(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        self.perform_sub_with_carry(destiny_value, byte as u16);
    }

    #[inline]
    pub(crate) fn execute_cmp_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
    }

    pub(crate) fn execute_daa(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let least_significant = destiny_value & 0x0f;
        let mut result = destiny_value;
        if least_significant > 9 || self.flags.auxiliary_carry {
            result += 6;
            self.flags.auxiliary_carry = (result & 0x0f) < least_significant;
        }
        let most_significant = (result & 0xf0) >> 4;
        if most_significant > 9 || self.flags.carry {
            result = result | ((most_significant + 6) << 4);
            if result > 0xff {
                self.flags.carry = true;
            }
        }
        self.update_flags(result, false);
        self.save_to_a(result as u8);
    }

    pub(crate) fn execute_dad(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_hl_value() as u32;
        let source_value = match register_type {
            RegisterType::B => self.get_current_bc_value() as u32,
            RegisterType::D => self.get_current_de_value() as u32,
            RegisterType::H => self.get_current_hl_value() as u32,
            RegisterType::Sp => self.get_current_sp_value() as u32,
            _ => panic!("{} is not a valid DAD argument!", register_type.to_string()),
        };
        let result = destiny_value + source_value;
        self.flags.carry = result > 0xffff;
        self.save_to_single_register((result >> 8) as u8, &RegisterType::H);
        self.save_to_single_register(result as u8, &RegisterType::L);
    }

    pub(crate) fn execute_dcr_by_register(&mut self, register_type: &RegisterType) {
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_sub_without_carry(source_value, 1);
        self.save_to_single_register(new_value, register_type);
    }

    pub(crate) fn execute_dcr_by_memory(&mut self) {
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_sub_without_carry(source_value, 1);
        self.set_value_in_memory_at_hl(new_value);
    }

    pub(crate) fn execute_dcx(&mut self, register_type: &RegisterType) {
        self.perform_step_on_double_register(register_type, false);
    }

    pub(crate) fn execute_inr_by_register(&mut self, register_type: &RegisterType) {
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_add_without_carry(source_value, 1);
        self.save_to_single_register(new_value, register_type);
    }

    pub(crate) fn execute_inr_by_memory(&mut self) {
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_add_without_carry(source_value, 1);
        self.set_value_in_memory_at_hl(new_value);
    }

    pub(crate) fn execute_inx(&mut self, register_type: &RegisterType) {
        self.perform_step_on_double_register(register_type, true);
    }

    pub(crate) fn execute_sbb_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let carry = self.flags.carry as u8;
        let source_value = (self.get_current_single_register_value(register_type) + carry) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_sbb_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let carry = self.flags.carry as u8;
        let source_value = (self.get_value_in_memory_at_hl() + carry) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_sbi(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        let add = byte as u16 + self.flags.carry as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, add);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_sub_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_sub_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_sui(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, byte as u16);
        self.save_to_a(new_value);
    }

    #[inline]
    fn perform_add_with_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_add(destiny, source, true)
    }

    #[inline]
    fn perform_add_without_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_add(destiny, source, true)
    }

    #[inline]
    fn perform_step_on_double_register(&mut self, register_type: &RegisterType, inc: bool) {
        let destiny_value = match register_type {
            RegisterType::B => self.get_current_bc_value() as u32,
            RegisterType::D => self.get_current_de_value() as u32,
            RegisterType::H => self.get_current_hl_value() as u32,
            RegisterType::Sp => self.get_current_sp_value() as u32,
            _ => panic!("{} is not a valid INX argument!", register_type.to_string()),
        };
        let result = if inc { destiny_value+1 } else { destiny_value - 1 };
        match register_type {
            RegisterType::B => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::B);
                self.save_to_single_register(result as u8, &RegisterType::C);
            }
            RegisterType::D => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::D);
                self.save_to_single_register(result as u8, &RegisterType::E);
            }
            RegisterType::H => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::H);
                self.save_to_single_register(result as u8, &RegisterType::L);
            }
            RegisterType::Sp => self.save_to_double_register(result as u16, &RegisterType::Sp),
            _ => panic!("{} is not a valid INX argument!", register_type.to_string()),
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
        self.perform_sub(destiny, source, true)
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
        self.flags.auxiliary_carry = (destiny & 0x0f) + ((!source + 1) & 0x0f) > 0x0f;
    }

    #[inline]
    fn update_auxiliary_carry(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (source & 0x0f) > 0x0f;
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use cpu::cpu::ROM_MEMORY_LIMIT;
    use disassembler_8080::Instruction;

    #[test]
    fn it_should_execute_cpi() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x4a);
        cpu.execute_instruction(Instruction::Cpi { byte: 0x40 });
        assert_eq!(cpu.get_current_a_value(), 0x4a);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbi_without_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Sbi { byte: 0x01 });
        assert_eq!(cpu.get_current_a_value(), 0xff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbi_with_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0);
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Sbi { byte: 0x01 });
        assert_eq!(cpu.get_current_a_value(), 0xfe);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sui() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Sui { byte: 0x01 });
        assert_eq!(cpu.get_current_a_value(), 0xff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }
}