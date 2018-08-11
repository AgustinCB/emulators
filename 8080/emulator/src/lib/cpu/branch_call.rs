use cpu::cpu::{Cpu, RegisterType, State};
use cpu::helpers::{two_bytes_to_word, word_to_address};
use std::process::exit;

impl<'a> Cpu<'a> {
    pub(crate) fn execute_rst(&mut self, value: u8) {
        if self.interruptions_enabled {
            let low_byte = (value & 0x07) << 3;
            self.perform_call(0, low_byte);
            self.state = State::Running;
        }
    }

    pub(crate) fn execute_call(&mut self, high_byte: u8, low_byte: u8) {
        let address = two_bytes_to_word(high_byte, low_byte);
        if self.cp_m_compatibility && address == 5 {
            self.handle_cp_m_print();
        } else if self.cp_m_compatibility && address == 0 {
            exit(0);
        } else {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cc(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.carry {
            self.perform_call(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_cm(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.sign {
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
        if !self.flags.sign {
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
        self.memory[sp-1].set(address[1]);
        self.memory[sp-2].set(address[0]);
        self.save_to_double_register((sp - 2) as u16, &RegisterType::Sp);
    }

    #[inline]
    fn handle_cp_m_print(&mut self) {
        let c_value = self.get_current_single_register_value(&RegisterType::C);
        if c_value == 9 {
            self.print_de_to_screen();
        } else if c_value == 2 {
            self.print_e_value_to_screen();
        }
    }

    #[inline]
    fn print_e_value_to_screen(&mut self) {
        let e_value = self.get_current_single_register_value(&RegisterType::E);
        self.print_message(&['E' as u8, ' ' as u8, e_value]);
    }

    #[inline]
    fn print_de_to_screen(&mut self) {
        let mut address = (self.get_current_de_value() + 3) as usize; // Skip prefix
        let mut bytes: Vec<u8> = Vec::new();
        while (self.memory[address].get() as char) != '$' {
            bytes.push(self.memory[address].get());
            address += 1;
        }
        self.print_message(bytes.as_ref());
    }

    #[inline]
    fn print_message(&mut self, bytes: &[u8]) {
        match self.printer {
            Some(ref mut screen) => screen.print(bytes),
            _ => panic!("Screen not configured while in CP/M compatibility mode."),
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, RegisterType, ROM_MEMORY_LIMIT, Printer, State};
    use cpu::instruction::Instruction;

    #[test]
    fn it_should_execute_call() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.execute_instruction(Instruction::Call { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
    }

    #[test]
    fn it_should_print_when_executing_call_to_5_while_in_cp_m_compatibility_mode() {
        struct FakePrinter { res: String }
        impl Printer for FakePrinter {
            fn print(&mut self, bytes: &[u8]) {
                self.res = String::from_utf8_lossy(bytes).to_string();
            }
        }
        let screen = &mut (FakePrinter { res: "".to_string() });
        {
            let mut cpu = Cpu::new_cp_m_compatible([0; ROM_MEMORY_LIMIT], screen);
            cpu.pc = 0x2c03;
            cpu.save_to_single_register(9, &RegisterType::C);
            cpu.save_to_single_register(0, &RegisterType::D);
            cpu.save_to_single_register(0, &RegisterType::E);
            cpu.memory[3].set('4' as u8);
            cpu.memory[4].set('2' as u8);
            cpu.memory[5].set('$' as u8);
            cpu.execute_instruction(Instruction::Call { address: [0x05, 0x00] });
            assert_eq!(cpu.pc, 0x2c03);
        }
        assert_eq!(screen.res, "42");
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
    }

    #[test]
    fn it_should_execute_cm_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = true;
        cpu.execute_instruction(Instruction::Cm { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cm_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = false;
        cpu.execute_instruction(Instruction::Cm { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
    }

    #[test]
    fn it_should_execute_cp_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = false;
        cpu.execute_instruction(Instruction::Cp { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x3c00);
        assert_eq!(cpu.get_current_sp_value(), 0);
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
    }

    #[test]
    fn it_shouldnt_execute_cp_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(2, &RegisterType::Sp);
        cpu.pc = 0x2c03;
        cpu.flags.sign = true;
        cpu.execute_instruction(Instruction::Cp { address: [0x00, 0x3c] });
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
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
        assert_eq!(cpu.memory[0].get(), 0);
        assert_eq!(cpu.memory[1].get(), 0);
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
        assert_eq!(cpu.memory[0].get(), 0x03);
        assert_eq!(cpu.memory[1].get(), 0x2c);
    }
}