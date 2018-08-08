use cpu::cpu::Cpu;

impl Cpu {
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
        self.perform_jump(high_byte, low_byte);
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
    use cpu::Cpu;
    use cpu::cpu::ROM_MEMORY_LIMIT;

    #[test]
    fn it_should_execute_pchl() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_jmp(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_should_execute_jc_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = true;
        cpu.execute_jc(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jc_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.carry = false;
        cpu.execute_jc(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jm_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.sign = true;
        cpu.execute_jm(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jm_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.sign = false;
        cpu.execute_jm(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jnc_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.execute_jnc(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jnc_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.carry = true;
        cpu.execute_jnc(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jnz_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.zero = false;
        cpu.execute_jnz(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jnz_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.zero = true;
        cpu.execute_jnz(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jp_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.sign = false;
        cpu.execute_jp(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jp_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.sign = true;
        cpu.execute_jp(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jpe_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.parity = true;
        cpu.execute_jpe(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jpe_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.parity = false;
        cpu.execute_jpe(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jpo_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.parity = false;
        cpu.execute_jpo(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jpo_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.parity = true;
        cpu.execute_jpo(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn it_should_execute_jz_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.zero = true;
        cpu.execute_jz(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }

    #[test]
    fn it_shouldnt_execute_jz_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.pc = 0;
        cpu.flags.zero = false;
        cpu.execute_jz(0x3c, 0x03);
        assert_eq!(cpu.pc, 0);
    }
}