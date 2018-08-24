use cpu::{Cpu, InputDevice, OutputDevice};
use failure::Error;
use super::CpuError;
use super::Mos6502Instruction;

const AVAILABLE_MEMORY: usize = 0x10000;

struct Mos6502Cpu {
    memory: [u8; AVAILABLE_MEMORY],
}

impl Cpu<u8, Mos6502Instruction, CpuError> for Mos6502Cpu {
    fn execute_instruction(&mut self, instruction: Mos6502Instruction) -> Result<(), Error> {
        Ok(())
    }

    fn get_next_instruction_bytes(&self) -> &[u8] {
        &self.memory[0..1]
    }

    fn can_run(&self, instruction: &Mos6502Instruction) -> bool {
        false
    }

    fn is_done(&self) -> bool {
        false
    }

    fn add_input_device(&mut self, id: u8, device: Box<InputDevice>) {
    }

    fn add_output_device(&mut self, id: u8, device: Box<OutputDevice>) {
    }

    fn increase_pc(&mut self, steps: u8) {
    }

    fn get_cycles_from_one_condition
        (&self, instruction: &Mos6502Instruction, not_met: u8, met: u8) -> u8 {
        0
    }

    fn get_cycles_from_two_conditions
        (&self, instruction: &Mos6502Instruction, not_met: u8, first_met: u8, second_met: u8) -> u8 {
        0
    }
}