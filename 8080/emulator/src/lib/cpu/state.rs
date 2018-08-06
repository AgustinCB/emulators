use cpu::cpu::Cpu;

impl Cpu {
    #[inline]
    pub(crate) fn execute_cma(&mut self) {
        let destiny_value = self.get_current_a_value();
        self.save_to_a(!destiny_value);
    }

    #[inline]
    pub(crate) fn execute_cmc(&mut self) {
        self.flags.carry = !self.flags.carry;
    }

    #[inline]
    pub(crate) fn execute_stc(&mut self) {
        self.flags.carry = true;
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use cpu::cpu::ROM_MEMORY_LIMIT;

    #[test]
    fn it_should_set_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.execute_stc();
        assert!(cpu.flags.carry);
    }

    #[test]
    fn it_should_invert_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.execute_cmc();
        assert!(cpu.flags.carry);
        cpu.execute_cmc();
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn it_should_complement_the_accumulator() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(42);
        cpu.execute_cma();
        assert_eq!(213, cpu.get_current_a_value());
    }
}