use cpu::Cpu;
use failure::Error;
use std::cmp::min;
use super::CpuError;
use super::Mos6502Instruction;
use super::instruction::Mos6502InstructionCode;

const AVAILABLE_MEMORY: usize = 0x10000;

struct Mos6502Cpu {
    memory: [u8; AVAILABLE_MEMORY],
    pc: u16,
}

impl Mos6502Cpu {
    #[inline]
    fn execute_nop(&self) {
    }
}

impl Cpu<u8, Mos6502Instruction, CpuError> for Mos6502Cpu {
    fn execute_instruction(&mut self, instruction: Mos6502Instruction) -> Result<(), Error> {
        if !self.can_run(&instruction) {
            return Ok(());
        }
        match instruction.instruction {
            Mos6502InstructionCode::Nop => self.execute_nop(),
            _ => self.execute_nop(),
        };
        Ok(())
    }

    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.pc as usize;
        let to = min(from+3, self.memory.len());
        &self.memory[from..to]
    }

    fn can_run(&self, _: &Mos6502Instruction) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.pc >= AVAILABLE_MEMORY as u16
    }

    fn increase_pc(&mut self, steps: u8) {
        self.pc += steps as u16
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