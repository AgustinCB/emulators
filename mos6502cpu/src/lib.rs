#[macro_use] extern crate cpu;
#[macro_use] extern crate failure;

mod alu;
mod bit_utils;
mod branch;
mod control;
mod data_shifting;
mod data_movement;
mod instruction;
mod logical;
mod math;
mod mos6502cpu;
mod stack;
mod undocumented;

pub type CpuResult = Result<(), CpuError>;

pub use mos6502cpu::{AVAILABLE_MEMORY, Memory, Mos6502Cpu, CpuError};
pub use cpu::{Cpu, Instruction};
pub use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode,
                      Mos6502InstructionError};