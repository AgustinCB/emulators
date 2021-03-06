use alloc::boxed::Box;
use alloc::fmt;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use super::cpu::{InputDevice, OutputDevice};
use super::CpuError;
use helpers::two_bytes_to_word;

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
            RegisterType::Sp => String::from("SP"),
            RegisterType::Psw => String::from("PSW"),
        };
        write!(f, "{}", s)
    }
}

pub type Address = [u8; 2];

#[derive(Debug, Fail)]
#[fail(display = "{} isn't a valid register.", register)]
pub struct LocationParsingError {
    register: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Location {
    Register { register: RegisterType },
    Memory,
}

impl ToString for Location {
    fn to_string(&self) -> String {
        match self {
            Location::Register { register } => register.to_string(),
            Location::Memory => String::from("M"),
        }
    }
}

impl Location {
    pub fn from(location: &str) -> Result<Self, LocationParsingError> {
        match location {
            "A" => Ok(Location::Register {
                register: RegisterType::A,
            }),
            "B" => Ok(Location::Register {
                register: RegisterType::B,
            }),
            "C" => Ok(Location::Register {
                register: RegisterType::C,
            }),
            "D" => Ok(Location::Register {
                register: RegisterType::D,
            }),
            "E" => Ok(Location::Register {
                register: RegisterType::E,
            }),
            "H" => Ok(Location::Register {
                register: RegisterType::H,
            }),
            "L" => Ok(Location::Register {
                register: RegisterType::L,
            }),
            "M" => Ok(Location::Memory),
            "SP" => Ok(Location::Register {
                register: RegisterType::Sp,
            }),
            "PSW" => Ok(Location::Register {
                register: RegisterType::Psw,
            }),
            _ => Err(LocationParsingError {
                register: String::from(location),
            }),
        }
    }
}

#[derive(Debug)]
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
        RegisterSet {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xffff,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum State {
    Running,
    Stopped,
    HardStop,
    Halted,
}

pub trait Printer {
    fn print(&mut self, bytes: &[u8]);
}

#[derive(Debug)]
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

pub struct Intel8080Cpu<'a> {
    pub(crate) registers: RegisterSet,
    pub(crate) pc: u16,
    pub memory: [u8; ROM_MEMORY_LIMIT * 8],
    pub(crate) cp_m_compatibility: bool,
    pub(crate) flags: Flags,
    pub interruptions_enabled: bool,
    pub(crate) state: State,
    pub(crate) prev_state: State,
    pub(crate) inputs: Vec<Option<Box<dyn InputDevice>>>,
    pub(crate) outputs: Vec<Option<Box<dyn OutputDevice>>>,
    pub(crate) printer: Option<&'a mut dyn Printer>,
}

impl<'a> Intel8080Cpu<'a> {
    pub fn new_cp_m_compatible(
        rom_memory: [u8; ROM_MEMORY_LIMIT],
        screen: &mut dyn Printer,
    ) -> Intel8080Cpu {
        let mut cpu = Intel8080Cpu::new(rom_memory);
        cpu.cp_m_compatibility = true;
        cpu.printer = Some(screen);
        cpu
    }

    pub fn new<'b>(rom_memory: [u8; ROM_MEMORY_LIMIT]) -> Intel8080Cpu<'b> {
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

        Intel8080Cpu {
            registers,
            pc: 0,
            memory,
            flags: Flags::new(),
            interruptions_enabled: true,
            state: State::Running,
            prev_state: State::Running,
            inputs: Intel8080Cpu::make_inputs_vector(),
            outputs: Intel8080Cpu::make_outputs_vector(),
            cp_m_compatibility: false,
            printer: None,
        }
    }

    pub fn get_debug_string(&self) -> String {
        let registers_string = alloc::format!("{:?}", self.registers)
            .replace("{", "{\n  ")
            .replace("}", "\n}");
        let flags_string = alloc::format!("{:?}", self.flags)
            .replace("true", "t")
            .replace("false", "f")
            .replace(", aux", ",\n  aux")
            .replace("{", "{\n  ")
            .replace("}", "\n}");
        alloc::format!("PC: {:?}\n{}\n{}", self.pc, &registers_string, &flags_string)
    }

    fn make_inputs_vector() -> Vec<Option<Box<dyn InputDevice>>> {
        let mut v = Vec::with_capacity(MAX_INPUT_OUTPUT_DEVICES);
        for _ in 0..MAX_INPUT_OUTPUT_DEVICES {
            v.push(None);
        }
        v
    }

    fn make_outputs_vector() -> Vec<Option<Box<dyn OutputDevice>>> {
        let mut v = Vec::with_capacity(MAX_INPUT_OUTPUT_DEVICES);
        for _ in 0..MAX_INPUT_OUTPUT_DEVICES {
            v.push(None);
        }
        v
    }

    pub fn is_hard_stopped(&self) -> bool {
        match self.state {
            State::HardStop => true,
            _ => false,
        }
    }

    pub fn toggle_hard_stop(&mut self) {
        match self.state {
            State::HardStop => {
                self.state = self.prev_state;
            }
            _ => {
                self.prev_state = self.state;
                self.state = State::HardStop;
            }
        }
    }

    #[inline]
    pub(crate) fn update_flags(&mut self, answer: u16, with_carry: bool) {
        self.flags.zero = answer.trailing_zeros() >= 8;
        self.flags.sign = (answer & 0x80) != 0;
        if with_carry {
            self.flags.carry = answer > 0xff;
        }
        self.flags.parity = (answer as u8).count_ones() % 2 == 0;
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
        self.get_current_single_register_value(RegisterType::A)
    }

    #[inline]
    pub(crate) fn get_current_sp_value(&self) -> u16 {
        self.registers.sp
    }

    #[inline]
    pub(crate) fn get_current_single_register_value(
        &self,
        register: RegisterType,
    ) -> Result<u8, CpuError> {
        match register {
            RegisterType::A => Ok(self.registers.a),
            RegisterType::B => Ok(self.registers.b),
            RegisterType::C => Ok(self.registers.c),
            RegisterType::D => Ok(self.registers.d),
            RegisterType::E => Ok(self.registers.e),
            RegisterType::H => Ok(self.registers.h),
            RegisterType::L => Ok(self.registers.l),
            _ => Err(CpuError::VirtualRegister { register }),
        }
    }

    #[inline]
    pub(crate) fn save_to_a(&mut self, new_value: u8) -> Result<(), CpuError> {
        self.save_to_single_register(new_value, RegisterType::A)
    }

    #[inline]
    pub(crate) fn save_to_sp(&mut self, new_value: u16) {
        self.registers.sp = new_value;
    }

    #[inline]
    pub(crate) fn save_to_single_register(
        &mut self,
        new_value: u8,
        register: RegisterType,
    ) -> Result<(), CpuError> {
        match register {
            RegisterType::A => {
                self.registers.a = new_value;
                Ok(())
            }
            RegisterType::B => {
                self.registers.b = new_value;
                Ok(())
            }
            RegisterType::C => {
                self.registers.c = new_value;
                Ok(())
            }
            RegisterType::D => {
                self.registers.d = new_value;
                Ok(())
            }
            RegisterType::E => {
                self.registers.e = new_value;
                Ok(())
            }
            RegisterType::H => {
                self.registers.h = new_value;
                Ok(())
            }
            RegisterType::L => {
                self.registers.l = new_value;
                Ok(())
            }
            _ => Err(CpuError::VirtualRegister { register }),
        }
    }

    #[inline]
    pub(crate) fn perform_jump(&mut self, high_byte: u8, low_byte: u8) {
        let new_pc = two_bytes_to_word(high_byte, low_byte);
        self.pc = new_pc;
    }

    pub(crate) fn execute_noop(&self) {}
}
