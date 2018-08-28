use intel8080cpu::{Intel8080Cpu, State};

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn execute_ei(&mut self) {
        self.interruptions_enabled = true;
    }

    pub(crate) fn execute_di(&mut self) {
        self.interruptions_enabled = false;
    }

    pub(crate) fn execute_hlt(&mut self) {
        self.state = State::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use intel8080cpu::{Intel8080Cpu, State, ROM_MEMORY_LIMIT};
    use instruction::Intel8080Instruction;
    use super::super::cpu::Cpu;

    #[test]
    fn it_should_execute_ei() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = false;
        cpu.execute_instruction(&Intel8080Instruction::Ei).unwrap();
        assert!(cpu.interruptions_enabled);
    }

    #[test]
    fn it_shouldnt_execute_ei_when_enabled() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = true;
        cpu.execute_instruction(&Intel8080Instruction::Ei).unwrap();
        assert!(cpu.interruptions_enabled);
    }

    #[test]
    fn it_should_execute_di() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = true;
        cpu.execute_instruction(&Intel8080Instruction::Di).unwrap();
        assert!(!cpu.interruptions_enabled);
    }

    #[test]
    fn it_shouldnt_execute_di_when_disabled() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.interruptions_enabled = false;
        cpu.execute_instruction(&Intel8080Instruction::Di).unwrap();
        assert!(!cpu.interruptions_enabled);
    }

    #[test]
    fn it_should_execute_hlt() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.state = State::Running;
        cpu.execute_instruction(&Intel8080Instruction::Hlt).unwrap();
        assert_eq!(cpu.state, State::Stopped);
    }
}