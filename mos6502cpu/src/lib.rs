#[macro_use] extern crate cpu;
#[macro_use] extern crate failure;

mod instruction;
mod math;
mod mos6502cpu;

pub type CpuResult = Result<(), CpuError>;

pub use mos6502cpu::{AVAILABLE_MEMORY, Mos6502Cpu, CpuError};
pub use cpu::Instruction;
pub use instruction::{Mos6502Instruction, Mos6502InstructionError};