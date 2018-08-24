use cpu::Cpu;
use failure::Error;
use std::cmp::min;
use super::CpuError;
use super::Mos6502Instruction;
use super::instruction::Mos6502InstructionCode;

const AVAILABLE_MEMORY: usize = 0x10000;

struct ProcessorStatus {
    negative: bool,
    overflow: bool,
    break_flag: bool,
    decimal: bool,
    interrupt: bool,
    zero: bool,
    carry: bool,
}

impl ProcessorStatus {
    fn new() -> ProcessorStatus {
        ProcessorStatus {
            negative: false,
            overflow: false,
            break_flag: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false,
        }
    }
}

struct RegisterSet {
    pc: u16,
    s: u8,
    x: u8,
    y: u8,
    a: u8,
    p: ProcessorStatus,
}

impl RegisterSet {
    fn new() -> RegisterSet {
        RegisterSet {
            pc: 0,
            s: 0xff,
            x: 0,
            y: 0,
            a: 0,
            p: ProcessorStatus::new(),
        }
    }
}

pub struct Mos6502Cpu {
    memory: [u8; AVAILABLE_MEMORY],
    registers: RegisterSet,
}

impl Mos6502Cpu {
    pub fn new(memory: [u8; AVAILABLE_MEMORY]) -> Mos6502Cpu {
        Mos6502Cpu {
            memory,
            registers: RegisterSet::new(),
        }
    }
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
        let from = self.registers.pc as usize;
        let to = min(from+3, self.memory.len());
        &self.memory[from..to]
    }

    fn can_run(&self, _: &Mos6502Instruction) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.registers.pc >= AVAILABLE_MEMORY as u16
    }

    fn increase_pc(&mut self, steps: u8) {
        self.registers.pc += steps as u16
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