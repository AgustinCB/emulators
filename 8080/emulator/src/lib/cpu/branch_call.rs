use cpu::cpu::{Cpu, RegisterType, State};
use cpu::helpers::word_to_address;

impl<'a> Cpu<'a> {
    pub(crate) fn execute_rst(&mut self, value: u8) {
        if self.interruptions_enabled {
            let low_byte = (value & 0x07) << 3;
            self.perform_call(0, low_byte);
            self.state = State::Running;
        }
    }

    pub(crate) fn execute_call(&mut self, high_byte: u8, low_byte: u8) {
        self.perform_call(high_byte, low_byte);
    }

    pub(crate) fn execute_cc(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.carry {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cm(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.sign {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cnc(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.carry {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cnz(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.zero {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cp(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.sign {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cpe(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.parity {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cpo(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.parity {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cz(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.zero {
            self.perform_call(high_byte, low_byte);
        }
    }

    #[inline]
    fn perform_call(&mut self, high_byte: u8, low_byte: u8) {
        self.push_program_counter_to_stack();
        self.perform_jump(high_byte, low_byte);
    }

    #[inline]
    fn push_program_counter_to_stack(&mut self) {
        let sp = self.get_current_sp_value() as usize;
        let address = word_to_address(self.pc);
        self.memory[sp-1] = address[1];
        self.memory[sp-2] = address[0];
        self.save_to_double_register((sp - 2) as u16, &RegisterType::Sp);
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, RegisterType, ROM_MEMORY_LIMIT, State};
    use cpu::instruction::Instruction;

    #[test]
    fn it_should_execute_call() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.execute_instruction(Instruction::Call { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_should_execute_cc_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Cc { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cc_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Cc { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cm_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = false;
        cpu.execute_instruction(Instruction::Cm { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cm_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = true;
        cpu.execute_instruction(Instruction::Cm { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cnc_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Cnc { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cnc_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Cnc { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cnz_if_zero_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.zero = false;
        cpu.execute_instruction(Instruction::Cnz { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cnz_if_zero_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.zero = true;
        cpu.execute_instruction(Instruction::Cnz { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cp_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = true;
        cpu.execute_instruction(Instruction::Cp { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cp_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = false;
        cpu.execute_instruction(Instruction::Cp { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cpe_if_parity_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.parity = true;
        cpu.execute_instruction(Instruction::Cpe { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cpe_if_parity_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.parity = false;
        cpu.execute_instruction(Instruction::Cpe { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cpo_if_parity_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.parity = false;
        cpu.execute_instruction(Instruction::Cpo { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cpo_if_parity_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.parity = true;
        cpu.execute_instruction(Instruction::Cpo { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_cz_if_zero_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.zero = true;
        cpu.execute_instruction(Instruction::Cz { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cz_if_zero_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.zero = false;
        cpu.execute_instruction(Instruction::Cz { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_rst_with_interruptions_enabled() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.execute_instruction(Instruction::Rst { value: 3 });
        assert_eq!(cpu.pc, 0x18);
        assert_eq!(cpu.state, State::Running);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_rst_with_interruptions_disabled() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.interruptions_enabled = false;
        cpu.execute_instruction(Instruction::Rst { value: 3 });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.state, State::Running);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0], 0);
        assert_eq!(cpu.memory[1], 0);
    }

    #[test]
    fn it_should_execute_rst_and_restart_cpu_when_stopped() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.state = State::Stopped;
        cpu.execute_instruction(Instruction::Rst { value: 3 });
        assert_eq!(cpu.pc, 0x18);
        assert_eq!(cpu.state, State::Running);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0], 0x03);
        assert_eq!(cpu.memory[1], 0x2c);
    }
}