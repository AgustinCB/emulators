use helpers::{bit_count, two_bytes_to_word};
use std::boxed::Box;
use std::fmt;
use super::CpuError;

pub const ROM_MEMORY_LIMIT: usize = 8192;
pub(crate) const MAX_INPUT_OUTPUT_DEVICES: usize = 0x100;
pub const HERTZ: i64 = 2_000_000;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum RegisterType {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Sp,
    Psw,
}

impl fmt::Display for RegisterType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            RegisterType::A => String::from("A"),
            RegisterType::B => String::from("B"),
            RegisterType::C => String::from("C"),
            RegisterType::D => String::from("D"),
            RegisterType::E => String::from("E"),
            RegisterType::H => String::from("H"),
            RegisterType::L => String::from("L"),
            RegisterType::Sp => String::from("Sp"),
            RegisterType::Psw => String::from("Psw"),
        };
        write!(f, "{}", s)
    }
}

pub type Address = [u8; 2];

#[derive(Clone, Copy)]
pub enum Location {
    Register { register: RegisterType },
    Memory,
}

impl ToString for Location {
    fn to_string(&self) -> String{
        match self {
            Location::Register { register } => register.to_string(),
            Location::Memory => String::from("(HL)")
        }
    }
}

pub(crate) struct RegisterSet {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
}

impl RegisterSet {
    pub(crate) fn new() -> RegisterSet {
        RegisterSet { a: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, sp: 0 }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum State {
    Running,
    Stopped,
}

pub trait InputDevice {
    fn read(&mut self) -> u8;
}

pub trait OutputDevice {
    fn write(&mut self, byte: u8);
}

pub trait Printer {
    fn print (&mut self, bytes: &[u8]);
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

pub struct Cpu<'a> {
    pub(crate) registers: RegisterSet,
    pub(crate) pc: u16,
    pub memory: [u8; ROM_MEMORY_LIMIT * 8],
    pub(crate) cp_m_compatibility: bool,
    pub(crate) flags: Flags,
    pub interruptions_enabled: bool,
    pub(crate) state: State,
    pub(crate) inputs: Vec<Option<Box<InputDevice>>>,
    pub(crate) outputs: Vec<Option<Box<OutputDevice>>>,
    pub(crate) printer: Option<&'a mut Printer>,
}

impl<'a> Cpu<'a> {
    pub fn new_cp_m_compatible(rom_memory: [u8; ROM_MEMORY_LIMIT], screen: &mut Printer) -> Cpu {
        let mut cpu = Cpu::new(rom_memory);
        cpu.cp_m_compatibility = true;
        cpu.printer = Some(screen);
        cpu
    }

    pub fn new<'b>(rom_memory: [u8; ROM_MEMORY_LIMIT]) -> Cpu<'b> {
        let registers = RegisterSet::new();
        let mut memory = [0; ROM_MEMORY_LIMIT * 8];
        for i in 0..(ROM_MEMORY_LIMIT * 8) {
            let value = if i < rom_memory.len() {
                rom_memory[i]
            } else {
                0
            };
            memory[i] = value;
        }

        Cpu {
            registers,
            pc: 0,
            memory,
            flags: Flags::new(),
            interruptions_enabled: true,
            state: State::Running,
            inputs: Cpu::make_inputs_vector(),
            outputs: Cpu::make_outputs_vector(),
            cp_m_compatibility: false,
            printer: None,
        }
    }

    fn make_inputs_vector() -> Vec<Option<Box<InputDevice>>> {
        let mut v = Vec::with_capacity(MAX_INPUT_OUTPUT_DEVICES);
        for _ in 0..MAX_INPUT_OUTPUT_DEVICES {
            v.push(None);
        }
        v
    }

    fn make_outputs_vector() -> Vec<Option<Box<OutputDevice>>> {
        let mut v = Vec::with_capacity(MAX_INPUT_OUTPUT_DEVICES);
        for _ in 0..MAX_INPUT_OUTPUT_DEVICES {
            v.push(None);
        }
        v
    }

    pub fn is_done(&self) -> bool {
        (self.pc as usize) >= ROM_MEMORY_LIMIT
    }

    pub fn add_input_device(&mut self, id: u8, device: Box<InputDevice>) {
        self.inputs[id as usize] = Some(device);
    }

    pub fn add_output_device(&mut self, id: u8, device: Box<OutputDevice>) {
        self.outputs[id as usize] = Some(device);
    }

    #[inline]
    pub(crate) fn update_flags(&mut self, answer: u16, with_carry: bool) {
        self.flags.zero = (answer & 0xff) == 0;
        self.flags.sign = (answer & 0x80) != 0;
        if with_carry {
            self.flags.carry = answer > 0xff;
        }
        self.flags.parity = bit_count(answer as u8) % 2 == 0;
    }

    #[inline]
    pub(crate) fn get_current_hl_value(&self) -> u16 {
        let high_value = self.registers.h;
        let low_value = self.registers.l;
        two_bytes_to_word(high_value, low_value)
    }

    #[inline]
    pub(crate) fn get_current_bc_value(&self) -> u16 {
        let high_value = self.registers.b;
        let low_value = self.registers.c;
        two_bytes_to_word(high_value, low_value)
    }

    #[inline]
    pub(crate) fn get_current_de_value(&self) -> u16 {
        let high_value = self.registers.d;
        let low_value = self.registers.e;
        two_bytes_to_word(high_value, low_value)
    }

    #[inline]
    pub(crate) fn get_value_in_memory_at_hl(&self) -> u8 {
        let source_value_address: u16 = self.get_current_hl_value();
        self.memory[source_value_address as usize]
    }

    #[inline]
    pub(crate) fn set_value_in_memory_at_hl(&mut self, value: u8) {
        let source_value_address: u16 = self.get_current_hl_value();
        self.memory[source_value_address as usize] = value;
    }

    #[inline]
    pub(crate) fn get_current_a_value(&self) -> Result<u8, CpuError> {
        self.get_current_single_register_value(&RegisterType::A)
    }

    #[inline]
    pub(crate) fn get_current_sp_value(&self) -> u16 {
        self.registers.sp
    }

    #[inline]
    pub(crate) fn get_current_single_register_value(&self, register: &RegisterType)
        -> Result<u8, CpuError> {
        match register {
            RegisterType::A => Ok(self.registers.a),
            RegisterType::B => Ok(self.registers.b),
            RegisterType::C => Ok(self.registers.c),
            RegisterType::D => Ok(self.registers.d),
            RegisterType::E => Ok(self.registers.e),
            RegisterType::H => Ok(self.registers.h),
            RegisterType::L => Ok(self.registers.l),
            _ => Err(CpuError::VirtualRegister { register: *register }),
        }
    }

    #[inline]
    pub(crate) fn save_to_a(&mut self, new_value: u8) -> Result<(), CpuError> {
        self.save_to_single_register(new_value, &RegisterType::A)
    }

    #[inline]
    pub(crate) fn save_to_sp(&mut self, new_value: u16) {
        self.registers.sp = new_value;
    }

    #[inline]
    pub(crate) fn save_to_single_register(&mut self, new_value: u8, register: &RegisterType)
        -> Result<(), CpuError> {
        match register {
            RegisterType::A => {
                self.registers.a = new_value;
                Ok(())
            },
            RegisterType::B => {
                self.registers.b = new_value;
                Ok(())
            },
            RegisterType::C => {
                self.registers.c = new_value;
                Ok(())
            },
            RegisterType::D => {
                self.registers.d = new_value;
                Ok(())
            },
            RegisterType::E => {
                self.registers.e = new_value;
                Ok(())
            },
            RegisterType::H => {
                self.registers.h = new_value;
                Ok(())
            },
            RegisterType::L => {
                self.registers.l = new_value;
                Ok(())
            },
            _ => Err(CpuError::VirtualRegister { register: *register }),
        }
    }

    #[inline]
    pub(crate) fn perform_jump(&mut self, high_byte: u8, low_byte: u8) {
        let new_pc = two_bytes_to_word(high_byte, low_byte);
        self.pc = new_pc;
    }
}
