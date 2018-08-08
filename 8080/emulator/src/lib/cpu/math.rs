use cpu::cpu::Cpu;
use disassembler_8080::RegisterType;

impl Cpu {
    pub(crate) fn execute_aci(&mut self, byte: u8) {
        let carry_as_u16 = self.flags.carry as u16;
        let destiny_value = (self.get_current_a_value() as u16 + carry_as_u16) & 0xff;
        let new_value = self.perform_add_with_carry(byte as u16, destiny_value);
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
    pub(crate) fn execute_cmp_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
    }

    #[inline]
    pub(crate) fn execute_cpi(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        self.perform_sub_with_carry(destiny_value, byte as u16);
    }

    // This is instruction has no tests because I'm not sure I even understand how it works.
    // It also seems to not be used... So I'll just ignore it for now.
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
        let source_value =
            ((self.get_current_single_register_value(register_type) + carry) & 0xff) as u16;
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
        self.perform_add(destiny, source, false)
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
    use disassembler_8080::{Instruction, Location, RegisterType};

    #[test]
    fn it_should_execute_aci_without_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.save_to_a(0x56);
        cpu.execute_instruction(Instruction::Aci { byte: 0xbe });
        assert_eq!(cpu.get_current_a_value(), 0x14);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_aci_with_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = true;
        cpu.save_to_a(0x14);
        cpu.execute_instruction(Instruction::Aci { byte: 0x42 });
        assert_eq!(cpu.get_current_a_value(), 0x57);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_adi() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x56);
        cpu.execute_instruction(Instruction::Adi { byte: 0xbe });
        assert_eq!(cpu.get_current_a_value(), 0x14);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_adc_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42);
        cpu.save_to_single_register(0x3d, &RegisterType::C);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Adc {
            source: Location::Register { register: RegisterType::C },
        });
        assert_eq!(cpu.get_current_a_value(), 0x7f);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_adc_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42);
        cpu.save_to_single_register(0x0, &RegisterType::H);
        cpu.save_to_single_register(0x0, &RegisterType::L);
        cpu.memory[0] = 0x3d;
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Adc { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x80);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_add_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x21);
        cpu.execute_instruction(Instruction::Add {
            source: Location::Register { register: RegisterType::A },
        });
        assert_eq!(cpu.get_current_a_value(), 0x42);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_add_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x6c);
        cpu.save_to_single_register(0x0, &RegisterType::H);
        cpu.save_to_single_register(0x0, &RegisterType::L);
        cpu.memory[0] = 0x2e;
        cpu.execute_instruction(Instruction::Add { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x9a);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_cmp_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x0a);
        cpu.save_to_single_register(0x05, &RegisterType::E);
        cpu.execute_instruction(Instruction::Cmp {
            source: Location::Register { register: RegisterType::E}
        });
        assert_eq!(cpu.get_current_a_value(), 0x0a);
        assert_eq!(cpu.get_current_single_register_value(&RegisterType::E), 0x05);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_cmp_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x02);
        cpu.save_to_single_register(0x0, &RegisterType::H);
        cpu.save_to_single_register(0x0, &RegisterType::L);
        cpu.memory[0] = 0x05;
        cpu.execute_instruction(Instruction::Cmp { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x02);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

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
    fn it_should_execute_dad() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x33, &RegisterType::B);
        cpu.save_to_single_register(0x9f, &RegisterType::C);
        cpu.save_to_single_register(0xa1, &RegisterType::H);
        cpu.save_to_single_register(0x7b, &RegisterType::L);
        cpu.execute_instruction(Instruction::Dad { register: RegisterType::B });
        assert_eq!(cpu.get_current_hl_value(), 0xd51a);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcr_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x40);
        cpu.execute_instruction(Instruction::Dcr {
            source: Location::Register { register: RegisterType::A },
        });
        assert_eq!(cpu.get_current_a_value(), 0x3f);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcr_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x3a, &RegisterType::H);
        cpu.save_to_single_register(0x7c, &RegisterType::L);
        cpu.memory[0x3a7c] = 0x40;
        cpu.execute_instruction(Instruction::Dcr { source: Location::Memory });
        assert_eq!(cpu.memory[0x3a7c], 0x3f);
        assert!(cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_dcx() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x98, &RegisterType::H);
        cpu.save_to_single_register(0x00, &RegisterType::L);
        cpu.execute_instruction(Instruction::Dcx { register: RegisterType::H });
        assert_eq!(cpu.get_current_hl_value(), 0x97ff);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_inr_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x99, &RegisterType::C);
        cpu.execute_instruction(Instruction::Inr {
            source: Location::Register { register: RegisterType::C },
        });
        assert_eq!(cpu.get_current_single_register_value(&RegisterType::C), 0x9a);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_inr_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x3a, &RegisterType::H);
        cpu.save_to_single_register(0x7c, &RegisterType::L);
        cpu.memory[0x3a7c] = 0x99;
        cpu.execute_instruction(Instruction::Inr { source: Location::Memory });
        assert_eq!(cpu.memory[0x3a7c], 0x9a);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_inx() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x38, &RegisterType::D);
        cpu.save_to_single_register(0xff, &RegisterType::E);
        cpu.execute_instruction(Instruction::Inx { register: RegisterType::D });
        assert_eq!(cpu.get_current_de_value(), 0x3900);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbb_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x04);
        cpu.save_to_single_register(0x02, &RegisterType::L);
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Sbb {
            source: Location::Register { register: RegisterType::L },
        });
        assert_eq!(cpu.get_current_a_value(), 0x01);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sbb_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x04);
        cpu.save_to_single_register(0x0, &RegisterType::H);
        cpu.save_to_single_register(0x0, &RegisterType::L);
        cpu.memory[0] = 0x02;
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Sbb { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x02);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
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
    fn it_should_execute_sub_by_register() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3e);
        cpu.execute_instruction(Instruction::Sub {
            source: Location::Register { register: RegisterType::A },
        });
        assert_eq!(cpu.get_current_a_value(), 0);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
        assert!(cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_sub_by_memory() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3e);
        cpu.save_to_single_register(0x0, &RegisterType::H);
        cpu.save_to_single_register(0x0, &RegisterType::L);
        cpu.memory[0] = 0x3d;
        cpu.execute_instruction(Instruction::Sub { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x01);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(cpu.flags.auxiliary_carry);
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