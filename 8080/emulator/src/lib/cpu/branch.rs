use cpu::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_pchl(&mut self) {
        let new_pc = self.get_current_hl_value();
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
        cpu.save_to_single_register(0x41, &RegisterType::H);
        cpu.save_to_single_register(0x3e, &RegisterType::L);
        cpu.execute_pchl();
        assert_eq!(cpu.pc, 0x413e);
    }
}