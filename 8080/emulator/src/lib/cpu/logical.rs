use cpu::cpu::Cpu;
use disassembler_8080::RegisterType;

impl Cpu {
    pub(crate) fn execute_ana_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_and(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    pub(crate) fn execute_ana_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_and(source_value, destiny_value);
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