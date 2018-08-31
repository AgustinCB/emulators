#[macro_use] extern crate cpu;
#[macro_use] extern crate failure;

use failure::Fail;

mod branch_call;
mod branch_jmp;
mod branch_ret;
mod intel8080cpu;
mod executable;
mod helpers;
mod instruction;
mod interruptions;
mod io;
mod logical;
mod math;
mod mov;
mod stack;
mod state;

#[derive(Debug, Fail)]
pub enum CpuError {
    #[fail(display = "Attempt to read from a device that doesn't exist: {}", id)]
    InputDeviceNotConfigured {
        id: u8,
    },
    #[fail(display = "This register is an invalid argument for that instruction: {}", register)]
    InvalidRegisterArgument {
        register: RegisterType,
    },
    #[fail(display = "You can't move data from (HL) to (HL)")]
    InvalidMemoryAccess,
    #[fail(display = "Attempt to write to a device that doesn't exist: {}", id)]
    OutputDeviceNotConfigured {
        id: u8,
    },
    #[fail(display = "This isn't a physical register: {}", register)]
    VirtualRegister {
        register: RegisterType,
    },
    #[fail(display = "The instruction doesn't support that kind of cycle calculation.")]
    InvalidCyclesCalculation,
}

pub use instruction::{Intel8080Instruction, Intel8080InstructionError};
pub use intel8080cpu::*;
pub use cpu::{Cpu, InputDevice, Instruction, OutputDevice, WithPorts};