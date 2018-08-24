#[macro_use] extern crate cpu;
#[macro_use] extern crate failure;

use failure::Fail;

#[derive(Debug, Fail)]
pub enum CpuError {
    #[fail(display = "Attempt to read from a device that doesn't exist: {}", id)]
    InputDeviceNotConfigured {
        id: u8,
    },
    #[fail(display = "Attempt to write to a device that doesn't exist: {}", id)]
    OutputDeviceNotConfigured {
        id: u8,
    },
}

mod instruction;
mod mos6502cpu;

pub use mos6502cpu::Mos6502Cpu;
pub use cpu::Instruction;
pub use instruction::{Mos6502Instruction, Mos6502InstructionError};