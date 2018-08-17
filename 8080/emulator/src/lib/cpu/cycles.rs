use cpu::cpu::Cpu;
use cpu::instruction::{Cycles, Instruction};

impl<'a> Cpu<'a> {
    pub(crate) fn get_cycles_for_instruction(&self, instruction: &Instruction) -> u8 {
        match instruction.get_cycles() {
            Cycles::Single(cycles) => cycles,
            Cycles::Conditional { not_met, met } =>
                self.get_cycles_from_condition(instruction, not_met, met),
        }
    }

    fn get_cycles_from_condition(&self, instruction: &Instruction, not_met: u8, met: u8) -> u8 {
        match instruction {
            Instruction::Cc { address: _} if self.flags.carry => met,
            Instruction::Cc { address: _} => not_met,
            Instruction::Cnc { address: _} if !self.flags.carry => met,
            Instruction::Cnc { address: _} => not_met,
            Instruction::Cz { address: _} if self.flags.zero => met,
            Instruction::Cz { address: _} => not_met,
            Instruction::Cnz { address: _} if !self.flags.zero => met,
            Instruction::Cnz { address: _} => not_met,
            Instruction::Cm { address: _} if self.flags.sign => met,
            Instruction::Cm { address: _} => not_met,
            Instruction::Cp { address: _} if !self.flags.sign => met,
            Instruction::Cp { address: _} => not_met,
            Instruction::Cpe { address: _} if self.flags.parity => met,
            Instruction::Cpe { address: _} => not_met,
            Instruction::Cpo { address: _} if !self.flags.parity => met,
            Instruction::Cpo { address: _} => not_met,
            Instruction::Rc if self.flags.carry => met,
            Instruction::Rc => not_met,
            Instruction::Rnc if !self.flags.carry => met,
            Instruction::Rnc => not_met,
            Instruction::Rz if self.flags.zero => met,
            Instruction::Rz => not_met,
            Instruction::Rnz if !self.flags.zero => met,
            Instruction::Rnz => not_met,
            Instruction::Rm if self.flags.sign => met,
            Instruction::Rm => not_met,
            Instruction::Rp if !self.flags.sign => met,
            Instruction::Rp => not_met,
            Instruction::Rpe if self.flags.parity => met,
            Instruction::Rpe => not_met,
            Instruction::Rpo if !self.flags.parity => met,
            Instruction::Rpo => not_met,
            _ => panic!("This instruction ({}) isn't conditional!", instruction.to_string()),
        }
    }
}