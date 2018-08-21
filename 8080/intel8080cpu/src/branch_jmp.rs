use cpu::Intel8080Cpu;
use helpers::two_bytes_to_word;
use std::process::exit;

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn execute_pchl(&mut self) {
        let new_pc = self.get_current_hl_value();
        self.pc = new_pc;
    }

    pub(crate) fn execute_jc(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.carry {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jmp(&mut self, high_byte: u8, low_byte: u8) {
        let address = two_bytes_to_word(high_byte, low_byte);
        if self.cp_m_compatibility && address == 0 {
            exit(0);
        } else {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jm(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.sign {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jnc(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.carry {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jnz(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.zero {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jp(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.sign {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jpe(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.parity {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jpo(&mut self, high_byte: u8, low_byte: u8) {
        if !self.flags.parity {
            self.perform_jump(high_byte, low_byte);
        }
    }

    pub(crate) fn execute_jz(&mut self, high_byte: u8, low_byte: u8) {
        if self.flags.zero {
            self.perform_jump(high_byte, low_byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::{Intel8080Cpu, ROM_MEMORY_LIMIT};
    use instruction::Intel8080Instruction;

    #[test]
    fn it_should_execute_pchl() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_instruction(Intel8080Instruction::Jmp { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_should_execute_jc_if_carry_is_set() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = true;
        cpu.execute_instruction(Intel8080Instruction::Jc { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jc_if_carry_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.carry = false;
        cpu.execute_instruction(Intel8080Instruction::Jc { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jm_if_sign_is_set() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.sign = true;
        cpu.execute_instruction(Intel8080Instruction::Jm { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jm_if_sign_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.sign = false;
        cpu.execute_instruction(Intel8080Instruction::Jm { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jnc_if_carry_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.execute_instruction(Intel8080Instruction::Jnc { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jnc_if_carry_is_set() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.carry = true;
        cpu.execute_instruction(Intel8080Instruction::Jnc { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jnz_if_zero_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.zero = false;
        cpu.execute_instruction(Intel8080Instruction::Jnz { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jnz_if_zero_is_set() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.zero = true;
        cpu.execute_instruction(Intel8080Instruction::Jnz { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jp_if_sign_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.sign = false;
        cpu.execute_instruction(Intel8080Instruction::Jp { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jp_if_sign_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.sign = true;
        cpu.execute_instruction(Intel8080Instruction::Jp { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jpe_if_parity_is_set() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.parity = true;
        cpu.execute_instruction(Intel8080Instruction::Jpe { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jpe_if_parity_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.parity = false;
        cpu.execute_instruction(Intel8080Instruction::Jpe { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jpo_if_parity_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.parity = false;
        cpu.execute_instruction(Intel8080Instruction::Jpo { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jpo_if_parity_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.parity = true;
        cpu.execute_instruction(Intel8080Instruction::Jpo { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jz_if_zero_is_set() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.zero = true;
        cpu.execute_instruction(Intel8080Instruction::Jz { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jz_if_zero_is_reset() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.zero = false;
        cpu.execute_instruction(Intel8080Instruction::Jz { address: [0x03, 0x3c] }).unwrap();
        assert_eq!(cpu.pc, 0);
    }
}