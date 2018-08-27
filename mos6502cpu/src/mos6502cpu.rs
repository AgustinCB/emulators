use cpu::Cpu;
use failure::{Error, Fail};
use std::cmp::min;
use super::Mos6502Instruction;
use super::instruction::{Mos6502InstructionCode, AddressingMode};

pub const AVAILABLE_MEMORY: usize = 0x10000;
const ZERO_PAGE_START: usize = 0;
const ZERO_PAGE_END: usize = 0x100;
const STACK_PAGE_START: usize = 0x100;
const STACK_PAGE_END: usize = 0x200;
const INTERRUPT_HANDLERS_START: usize = 0xFFFA;
const INTERRUPT_HANDLERS_END: usize = 0x10000;

fn two_bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    (high_byte as u16) << 8 | (low_byte as u16)
}

#[derive(Debug, Fail)]
pub enum CpuError {
    #[fail(display = "Attempt to access reserved memory. 0x0000-0x0200 and 0xFFFA to 0x10000 are reserved.")]
    ReservedMemory,
    #[fail(display = "Attempt to use invalid addressing mode.")]
    InvalidAddressingMode,
}

pub(crate) struct ProcessorStatus {
    pub(crate) negative: bool,
    pub(crate) overflow: bool,
    pub(crate) break_flag: bool,
    pub(crate) decimal: bool,
    pub(crate) interrupt: bool,
    pub(crate) zero: bool,
    pub(crate) carry: bool,
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

pub(crate) struct RegisterSet {
    pub(crate) pc: u16,
    pub(crate) s: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) a: u8,
    pub(crate) p: ProcessorStatus,
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
    pub(crate) registers: RegisterSet,
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

    pub(crate) fn get_value_from_addressing_mode(&self, addressing_mode: AddressingMode) -> u8 {
        match addressing_mode {
            AddressingMode::Immediate { byte } => byte,
            AddressingMode::ZeroPage { byte } => self.memory[byte as usize],
            AddressingMode::ZeroPageIndexedX { byte } => {
                let x = self.registers.x as u16;
                let address = (x + byte as u16) as u8;
                self.memory[address as usize]
            },
            AddressingMode::ZeroPageIndexedY { byte } => {
                let y = self.registers.y as u16;
                let address = (y + byte as u16) as u8;
                self.memory[address as usize]
            },
            AddressingMode::Absolute { high_byte, low_byte } => {
                let address = two_bytes_to_word(high_byte, low_byte) as usize;
                self.memory[address]
            },
            AddressingMode::AbsoluteIndexedX { high_byte, low_byte } => {
                let address = two_bytes_to_word(high_byte, low_byte) as usize;
                self.memory[address + self.registers.x as usize]
            },
            AddressingMode::AbsoluteIndexedY { high_byte, low_byte } => {
                let address = two_bytes_to_word(high_byte, low_byte) as usize;
                self.memory[address + self.registers.y as usize]
            },
            AddressingMode::IndexedIndirect { byte } => {
                let indirect_address = ((byte as u16 + self.registers.x as u16) as u8) as usize;
                let (low_byte, high_byte) =
                    (self.memory[indirect_address], self.memory[indirect_address+1]);
                self.memory[two_bytes_to_word(high_byte, low_byte) as usize]
            },
            AddressingMode::IndirectIndexed { byte } => {
                let (low_byte, high_byte) =
                    (self.memory[byte as usize], self.memory[byte as usize + 1]);
                let direct_address =
                    two_bytes_to_word(high_byte, low_byte) + self.registers.y as u16;
                self.memory[direct_address as usize]
            },
            _ => panic!("Not implemented yet"),
        }
    }
}

impl Cpu<u8, Mos6502Instruction, CpuError> for Mos6502Cpu {
    fn execute_instruction(&mut self, instruction: Mos6502Instruction) -> Result<(), Error> {
        if !self.can_run(&instruction) {
            return Ok(());
        }
        match instruction.instruction {
            Mos6502InstructionCode::Adc => self.execute_adc(instruction.addressing_mode)?,
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
    use instruction::AddressingMode;
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

    #[test]
    fn it_should_get_value_from_addressing_mode_for_immediate() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::Immediate { byte: 0x42 }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x35] = 0x42;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::ZeroPage { byte: 0x35 }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x20] = 0x42;
        cpu.registers.x = 0x60;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::ZeroPageIndexedX { byte: 0xC0 }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x20] = 0x42;
        cpu.registers.y = 0x60;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::ZeroPageIndexedY { byte: 0xC0 }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x2442] = 0x42;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x42,
            }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x2443] = 0x42;
        cpu.registers.x = 1;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x43,
            }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x2443] = 0x42;
        cpu.registers.y = 1;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x43,
            }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_indexed_indirect() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x24] = 0x74;
        cpu.memory[0x25] = 0x20;
        cpu.memory[0x2074] = 0x42;
        cpu.registers.x = 0x04;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::IndexedIndirect {
                byte: 0x20,
            }),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_indirect_indexed() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x86] = 0x28;
        cpu.memory[0x87] = 0x40;
        cpu.memory[0x4028] = 0x42;
        cpu.registers.y = 0x10;
        assert_eq!(
            cpu.get_value_from_addressing_mode(AddressingMode::IndexedIndirect {
                byte: 0x86,
            }),
            0x42);
    }
}