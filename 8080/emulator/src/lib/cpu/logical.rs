use cpu::cpu::{Cpu, RegisterType};

impl<'a> Cpu<'a> {
    pub(crate) fn execute_ana_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_and(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ana_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_and(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ani(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value();
        let new_value = self.perform_and(destiny_value, byte);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ora_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_or(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ora_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_or(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ori(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value();
        let new_value = self.perform_or(destiny_value, byte);
        self.save_to_a(new_value);
    }

    #[inline]
    pub(crate) fn execute_ral(&mut self) {
        let a_value = self.get_current_a_value();
        let operand  = if self.flags.carry {
            a_value | 0x80
        } else {
            a_value & (!0x80)
        };
        self.flags.carry = (a_value & 0x80) == 0x80;
        self.save_to_a(operand.rotate_left(1));
    }

    #[inline]
    pub(crate) fn execute_rar(&mut self) {
        let a_value = self.get_current_a_value();
        let new_a_value = if self.flags.carry {
            a_value.rotate_right(1) | 0x80
        } else {
            a_value.rotate_right(1) & (!0x80)
        };
        self.save_to_a(new_a_value);
        self.flags.carry = (a_value & 0x01) == 0x01;
    }

    #[inline]
    pub(crate) fn execute_rlc(&mut self) {
        let value = self.get_current_a_value().rotate_left(1);
        self.flags.carry = (value & 0x01) != 0;
        self.save_to_a(value);
    }

    #[inline]
    pub(crate) fn execute_rrc(&mut self) {
        let value = self.get_current_a_value().rotate_right(1);
        self.flags.carry = (value & 0x80) != 0;
        self.save_to_a(value);
    }

    pub(crate) fn execute_xra_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_xor(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_xra_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_xor(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_xri(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value();
        let new_value = self.perform_xor(destiny_value, byte);
        self.save_to_a(new_value);
    }

    #[inline]
    fn perform_and(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny & source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        answer
    }

    #[inline]
    fn perform_or(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny | source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        answer
    }

    #[inline]
    fn perform_xor(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny ^ source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
        answer
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, Location, RegisterType, ROM_MEMORY_LIMIT};
    use cpu::instruction::Instruction;

    #[test]
    fn it_should_execute_ana_by_memory () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xfc);
        cpu.save_to_single_register(0x00, &RegisterType::H);
        cpu.save_to_single_register(0x00, &RegisterType::L);
        cpu.memory[0].set(0x0f);
        cpu.execute_instruction(Instruction::Ana { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x0c);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ana_by_register () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xfc);
        cpu.save_to_single_register(0x0f, &RegisterType::C);
        cpu.execute_instruction(Instruction::Ana {
            source: Location::Register { register: RegisterType::C }
        });
        assert_eq!(cpu.get_current_a_value(), 0x0c);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ani() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3a);
        cpu.execute_instruction(Instruction::Ani { byte: 0x0f });
        assert_eq!(cpu.get_current_a_value(), 0x0a);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ora_by_memory () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x33);
        cpu.save_to_single_register(0x00, &RegisterType::H);
        cpu.save_to_single_register(0x00, &RegisterType::L);
        cpu.memory[0].set(0x0f);
        cpu.execute_instruction(Instruction::Ora { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x3f);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ora_by_register () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x33);
        cpu.save_to_single_register(0x0f, &RegisterType::C);
        cpu.execute_instruction(Instruction::Ora {
            source: Location::Register { register: RegisterType::C }
        });
        assert_eq!(cpu.get_current_a_value(), 0x3f);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ori() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xb5);
        cpu.execute_instruction(Instruction::Ori { byte: 0x0f });
        assert_eq!(cpu.get_current_a_value(), 0xbf);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_ral() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xb5);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Ral);
        assert_eq!(cpu.get_current_a_value(), 0x6a);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_rar() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x6a);
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Rar);
        assert_eq!(cpu.get_current_a_value(), 0xb5);
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_rlc() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xf2);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Rlc);
        assert_eq!(cpu.get_current_a_value(), 0xe5);
        assert!(cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_rrc() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xf2);
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Rrc);
        assert_eq!(cpu.get_current_a_value(), 0x79);
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn it_should_execute_xri() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3b);
        cpu.execute_instruction(Instruction::Xri { byte: 0x81 });
        assert_eq!(cpu.get_current_a_value(), 0xba);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(!cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_xra_by_memory () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x78);
        cpu.save_to_single_register(0x00, &RegisterType::H);
        cpu.save_to_single_register(0x00, &RegisterType::L);
        cpu.memory[0].set(0x5c);
        cpu.execute_instruction(Instruction::Xra { source: Location::Memory });
        assert_eq!(cpu.get_current_a_value(), 0x24);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_xra_by_register () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0xff);
        cpu.save_to_single_register(0x0f, &RegisterType::C);
        cpu.execute_instruction(Instruction::Xra {
            source: Location::Register { register: RegisterType::C }
        });
        assert_eq!(cpu.get_current_a_value(), 0xf0);
        assert!(!cpu.flags.carry);
        assert!(cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }

    #[test]
    fn it_should_execute_xra_on_itself () {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x33);
        cpu.execute_instruction(Instruction::Xra {
            source: Location::Register { register: RegisterType::A }
        });
        assert_eq!(cpu.get_current_a_value(), 0);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.zero);
    }
}