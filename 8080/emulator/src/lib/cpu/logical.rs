use cpu::cpu::Cpu;
use disassembler_8080::RegisterType;

impl Cpu {
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
        let new_value = self.perform_or(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ora_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_or(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    #[inline]
    pub(crate) fn execute_ral(&mut self) {
        let value = self.get_current_a_value().rotate_left(1);
        let carry_mask = if self.flags.carry {
            0x80
        } else {
            0
        };
        self.flags.carry = (value & 0x01) != 0;
        self.save_to_a(value | carry_mask);
    }

    #[inline]
    pub(crate) fn execute_rar(&mut self) {
        let value = self.get_current_a_value().rotate_right(1);
        let carry_mask = if self.flags.carry {
            0x80
        } else {
            0
        };
        self.flags.carry = (value & 0x80) != 0;
        self.save_to_a(value | carry_mask);
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
        let new_value = self.perform_xor(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_xra_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_xor(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    #[inline]
    fn perform_and(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny & source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        answer
    }

    #[inline]
    fn perform_or(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny | source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        answer
    }

    #[inline]
    fn perform_xor(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny ^ source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        answer
    }
}

#[cfg(tests)]
mod tests {
    use cpu::Cpu;
    use cpu::cpu::ROM_MEMORY_LIMIT;

    #[test]
    fn it_should_execute_ani() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x3a);
        cpu.execute_ani(0x0f);
        assert_eq!(cpu.get_current_a_value(), 0x0a);
        assert!(!cpu.flags.carry);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(!cpu.flags.zero);
    }
}