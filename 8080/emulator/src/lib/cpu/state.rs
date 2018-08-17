use cpu::cpu::Cpu;
use super::CpuError;

impl<'a> Cpu<'a> {
    #[inline]
    pub(crate) fn execute_cma(&mut self) -> Result<(), CpuError> {
        let destiny_value = self.get_current_a_value()?;
        self.save_to_a(!destiny_value)
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
    use cpu::cpu::{Cpu, ROM_MEMORY_LIMIT};
    use cpu::instruction::Instruction;

    #[test]
    fn it_should_set_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Stc).unwrap();
        assert!(cpu.flags.carry);
    }

    #[test]
    fn it_should_invert_carry() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Cmc).unwrap();
        assert!(cpu.flags.carry);
        cpu.execute_instruction(Instruction::Cmc).unwrap();
        assert!(!cpu.flags.carry);
    }

    #[test]
    fn it_should_complement_the_accumulator() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(42).unwrap();
        cpu.execute_instruction(Instruction::Cma).unwrap();
        assert_eq!(213, cpu.get_current_a_value().unwrap());
    }
}