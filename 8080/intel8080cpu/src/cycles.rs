use intel8080cpu::Intel8080Cpu;
use instruction::Intel8080Instruction;
use super::cpu::{Cycles, Instruction};

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn get_cycles_for_instruction(&self, instruction: &Intel8080Instruction) -> u8 {
        match instruction.get_cycles() {
            Cycles::Single(cycles) => cycles,
            Cycles::Conditional { not_met, met } =>
                self.get_cycles_from_condition(instruction, not_met, met),
        }
    }

    fn get_cycles_from_condition(&self, instruction: &Intel8080Instruction, not_met: u8, met: u8) -> u8 {
        match instruction {
            Intel8080Instruction::Cc { address: _} if self.flags.carry => met,
            Intel8080Instruction::Cc { address: _} => not_met,
            Intel8080Instruction::Cnc { address: _} if !self.flags.carry => met,
            Intel8080Instruction::Cnc { address: _} => not_met,
            Intel8080Instruction::Cz { address: _} if self.flags.zero => met,
            Intel8080Instruction::Cz { address: _} => not_met,
            Intel8080Instruction::Cnz { address: _} if !self.flags.zero => met,
            Intel8080Instruction::Cnz { address: _} => not_met,
            Intel8080Instruction::Cm { address: _} if self.flags.sign => met,
            Intel8080Instruction::Cm { address: _} => not_met,
            Intel8080Instruction::Cp { address: _} if !self.flags.sign => met,
            Intel8080Instruction::Cp { address: _} => not_met,
            Intel8080Instruction::Cpe { address: _} if self.flags.parity => met,
            Intel8080Instruction::Cpe { address: _} => not_met,
            Intel8080Instruction::Cpo { address: _} if !self.flags.parity => met,
            Intel8080Instruction::Cpo { address: _} => not_met,
            Intel8080Instruction::Rc if self.flags.carry => met,
            Intel8080Instruction::Rc => not_met,
            Intel8080Instruction::Rnc if !self.flags.carry => met,
            Intel8080Instruction::Rnc => not_met,
            Intel8080Instruction::Rz if self.flags.zero => met,
            Intel8080Instruction::Rz => not_met,
            Intel8080Instruction::Rnz if !self.flags.zero => met,
            Intel8080Instruction::Rnz => not_met,
            Intel8080Instruction::Rm if self.flags.sign => met,
            Intel8080Instruction::Rm => not_met,
            Intel8080Instruction::Rp if !self.flags.sign => met,
            Intel8080Instruction::Rp => not_met,
            Intel8080Instruction::Rpe if self.flags.parity => met,
            Intel8080Instruction::Rpe => not_met,
            Intel8080Instruction::Rpo if !self.flags.parity => met,
            Intel8080Instruction::Rpo => not_met,
            _ => panic!("This instruction ({}) isn't conditional!", instruction.to_string()),
        }
    }
}