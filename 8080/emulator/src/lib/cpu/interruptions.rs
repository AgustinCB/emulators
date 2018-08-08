use cpu::cpu::Cpu;

impl Cpu {
    pub(crate) fn execute_ei(&mut self) {
        self.interruptions_enabled = true;
    }

    pub(crate) fn execute_di(&mut self) {
        self.interruptions_enabled = false;
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, ROM_MEMORY_LIMIT};
    use disassembler_8080::Instruction;

    #[test]
    fn it_should_execute_ei() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = false;
        cpu.execute_instruction(Instruction::Ei);
        assert!(cpu.interruptions_enabled);
    }

    #[test]
    fn it_shouldnt_execute_ei_when_enabled() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = true;
        cpu.execute_instruction(Instruction::Ei);
        assert!(cpu.interruptions_enabled);
    }

    #[test]
    fn it_should_execute_di() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = true;
        cpu.execute_instruction(Instruction::Di);
        assert!(!cpu.interruptions_enabled);
    }

    #[test]
    fn it_shouldnt_execute_di_when_disabled() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = false;
        cpu.execute_instruction(Instruction::Di);
        assert!(!cpu.interruptions_enabled);
    }
}