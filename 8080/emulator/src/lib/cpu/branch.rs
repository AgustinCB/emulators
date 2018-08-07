use cpu::cpu::Cpu;
use cpu::helpers::two_bytes_to_word;

impl Cpu {
    pub(crate) fn execute_pchl(&mut self) {
        let new_pc = self.get_current_hl_value();
        self.pc = new_pc;
    }

    pub(crate) fn execute_jmp(&mut self, high_byte: u8, low_byte: u8) {
        let new_pc = two_bytes_to_word(high_byte, low_byte);
        self.pc = new_pc;
    }
}

#[cfg(tests)]
mod tests {
    use cpu::Cpu;
    use cpu::cpu::ROM_MEMORY_LIMIT;
    use disassembler_8080::RegisterType;

    #[test]
    fn it_should_execut_pchl() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_jmp(0x3c, 0x03);
        assert_eq!(cpu.pc, 0x3c03);
    }
}