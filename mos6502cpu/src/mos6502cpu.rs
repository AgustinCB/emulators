use cpu::Cpu;
use failure::{Error, Fail};
use std::cmp::min;
use super::Mos6502Instruction;
use super::instruction::Mos6502InstructionCode;

pub const AVAILABLE_MEMORY: usize = 0x10000;
const ZERO_PAGE_START: usize = 0;
const ZERO_PAGE_END: usize = 0x100;
const STACK_PAGE_START: usize = 0x100;
const STACK_PAGE_END: usize = 0x200;
const INTERRUPT_HANDLERS_START: usize = 0xFFFA;
const INTERRUPT_HANDLERS_END: usize = 0x10000;

#[derive(Debug, Fail)]
pub enum CpuError {
    #[fail(display = "Attempt to access reserved memory. 0x0000-0x0200 and 0xFFFA to 0x10000 are reserved.")]
    ReservedMemory,
}

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

    pub fn get_memory_slice(&mut self, from: usize, to: usize) -> Result<&[u8], CpuError> {
        if (from >= ZERO_PAGE_START && from < ZERO_PAGE_END) ||
            (from >= STACK_PAGE_START && from < STACK_PAGE_END) ||
            (from >= INTERRUPT_HANDLERS_START && from < INTERRUPT_HANDLERS_END) {
            Err(CpuError::ReservedMemory)
        } else if (to >= ZERO_PAGE_START && to < ZERO_PAGE_END) ||
            (to >= STACK_PAGE_START && to < STACK_PAGE_END) ||
            (to >= INTERRUPT_HANDLERS_START && to < INTERRUPT_HANDLERS_END) {
            Err(CpuError::ReservedMemory)
        } else if (from < ZERO_PAGE_START && to >= ZERO_PAGE_END) ||
            (from < STACK_PAGE_START && to >= STACK_PAGE_END) ||
            (from < INTERRUPT_HANDLERS_START && to >= INTERRUPT_HANDLERS_END) {
            Err(CpuError::ReservedMemory)
        } else {
            Ok(&mut self.memory[from..to])
        }

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

#[cfg(test)]
mod tests {
    use AVAILABLE_MEMORY;
    use Mos6502Cpu;

    #[test]
    fn it_shouldnt_access_zero_page_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        {
            let mem = cpu.get_memory_slice(0, 0x200);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0x20, 0x200);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0x20, 0x1FF);
            assert!(mem.is_err());
        }
    }

    #[test]
    fn it_shouldnt_access_stack_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        {
            let mem = cpu.get_memory_slice(0x100, 0x300);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFF, 0x1FA);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0x101, 0x1FA);
            assert!(mem.is_err());
        }
    }


    #[test]
    fn it_shouldnt_access_interrupt_handlers_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        {
            let mem = cpu.get_memory_slice(0x100, 0xFFFB);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFFFA, 0x10001);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFFF5, 0x10001);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFFFB, 0xFFFC);
            assert!(mem.is_err());
        }
    }
}