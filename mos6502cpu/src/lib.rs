#[macro_use] extern crate cpu;
#[macro_use] extern crate failure;

mod instruction;

pub use cpu::Instruction;
pub use instruction::Mos6502Instruction;