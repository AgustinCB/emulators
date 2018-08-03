extern crate disassembler_8080;

use self::disassembler_8080::RegisterType;
use std::collections::HashMap;

const ROM_MEMORY_LIMIT: usize = 8192;

pub(crate) enum Register {
    SingleRegister { value: u8 },
    DoubleRegister { value: u16 },
}

impl Register {
    fn new(t: RegisterType) -> Register {
        match t {
            RegisterType::Sp =>
                Register::DoubleRegister { value: 0 },
            _ => Register::SingleRegister { value: 0 },
        }
    }
}

pub(crate) struct Flags {
    pub(crate) sign: bool,
    pub(crate) zero: bool,
    pub(crate) parity: bool,
    pub(crate) carry: bool,
    pub(crate) auxiliary_carry: bool,
}

impl Flags {
    fn new() -> Flags {
        Flags {
            sign: true,
            zero: true,
            parity: true,
            carry: true,
            auxiliary_carry: true,
        }
    }
}

pub struct State {
    pub(crate) registers: HashMap<RegisterType, Register>,
    pub(crate) pc: u16,
    pub(crate) memory: [u8; ROM_MEMORY_LIMIT * 2],
    pub(crate) flags: Flags,
    pub(crate) interruptions_enabled: bool,
}

impl State {
    pub fn new(rom_memory: [u8; ROM_MEMORY_LIMIT]) -> State {
        let mut registers = HashMap::new();
        let mut memory = [0; ROM_MEMORY_LIMIT * 2];
        registers.insert(RegisterType::A, Register::new(RegisterType::A));
        registers.insert(RegisterType::B, Register::new(RegisterType::B));
        registers.insert(RegisterType::C, Register::new(RegisterType::C));
        registers.insert(RegisterType::D, Register::new(RegisterType::D));
        registers.insert(RegisterType::E, Register::new(RegisterType::E));
        registers.insert(RegisterType::H, Register::new(RegisterType::H));
        registers.insert(RegisterType::L, Register::new(RegisterType::L));
        registers.insert(RegisterType::Sp, Register::new(RegisterType::Sp));
        for i in 0..rom_memory.len() {
            memory[i] = rom_memory[i];
        }
        State {
            registers: registers,
            pc: 0,
            memory: memory,
            flags: Flags::new(),
            interruptions_enabled: true,
        }
    }

    pub fn is_done(&self) -> bool {
        (self.pc as usize) >= ROM_MEMORY_LIMIT
    }
}
