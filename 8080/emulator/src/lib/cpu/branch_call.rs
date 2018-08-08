use cpu::cpu::Cpu;
use cpu::helpers::word_to_address;
use disassembler_8080::RegisterType;

impl Cpu {
    pub(crate) fn execute_call(&mut self, high_byte: u8, low_byte: u8) {
        self.perform_call(high_byte, low_byte);
    }

    fn perform_call(&mut self, high_byte: u8, low_byte: u8) {
        self.push_program_counter_to_stack();
        self.perform_jump(high_byte, low_byte);
    }

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
    use cpu::cpu::Cpu;
    use cpu::cpu::ROM_MEMORY_LIMIT;
    use disassembler_8080::{Instruction, RegisterType};

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
}