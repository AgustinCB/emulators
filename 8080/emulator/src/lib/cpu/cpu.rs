use cpu::helpers::{bit_count, two_bytes_to_word};
use std::boxed::Box;
use std::cell::Cell;
use std::collections::HashMap;

pub const ROM_MEMORY_LIMIT: usize = 8192;
pub(crate) const MAX_INPUT_OUTPUT_DEVICES: usize = 0x100;
const NUM_REGISTERS: usize = 8;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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

impl ToString for RegisterType {
    fn to_string(&self) -> String {
        match self {
            RegisterType::A => String::from("A"),
            RegisterType::B => String::from("B"),
            RegisterType::C => String::from("C"),
            RegisterType::D => String::from("D"),
            RegisterType::E => String::from("E"),
            RegisterType::H => String::from("H"),
            RegisterType::L => String::from("L"),
            RegisterType::Sp => String::from("SP"),
            RegisterType::Psw => String::from("PSW"),
        }
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

pub(crate) enum Register {
    SingleRegister { value: u8 },
    DoubleRegister { value: u16 },
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

pub trait Screen {
    fn print (&mut self, bytes: &[u8]);
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

pub struct Cpu<'a> {
    pub(crate) registers: HashMap<RegisterType, Register>,
    pub(crate) pc: u16,
    pub memory: Vec<Cell<u8>>,
    pub(crate) cp_m_compatibility: bool,
    pub(crate) flags: Flags,
    pub(crate) interruptions_enabled: bool,
    pub(crate) state: State,
    pub(crate) inputs: Vec<Option<Box<InputDevice>>>,
    pub(crate) outputs: Vec<Option<Box<OutputDevice>>>,
    pub(crate) screen: Option<&'a mut Screen>,
}

impl<'a> Cpu<'a> {
    pub fn new_cp_m_compatible(rom_memory: [u8; ROM_MEMORY_LIMIT], screen: &mut Screen) -> Cpu {
        let mut cpu = Cpu::new(rom_memory);
        cpu.cp_m_compatibility = true;
        cpu.screen = Some(screen);
        cpu
    }

    pub fn new<'b>(rom_memory: [u8; ROM_MEMORY_LIMIT]) -> Cpu<'b> {
        let registers = Cpu::make_register_map();
        let mut memory = Vec::with_capacity(ROM_MEMORY_LIMIT * 8);
        for _ in 0..(ROM_MEMORY_LIMIT * 8) {
            memory.push(Cell::new(0));
        }
        for i in 0..rom_memory.len() {
            memory[i].set(rom_memory[i]);
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
            screen: None,
        }
    }

    fn make_register_map() -> HashMap<RegisterType, Register> {
        let mut registers = HashMap::with_capacity(NUM_REGISTERS);
        registers.insert(RegisterType::A, Register::new(RegisterType::A));
        registers.insert(RegisterType::B, Register::new(RegisterType::B));
        registers.insert(RegisterType::C, Register::new(RegisterType::C));
        registers.insert(RegisterType::D, Register::new(RegisterType::D));
        registers.insert(RegisterType::E, Register::new(RegisterType::E));
        registers.insert(RegisterType::H, Register::new(RegisterType::H));
        registers.insert(RegisterType::L, Register::new(RegisterType::L));
        registers.insert(RegisterType::Sp, Register::new(RegisterType::Sp));
        registers
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

    pub(crate) fn add_input_device(&mut self, id: u8, device: Box<InputDevice>) {
        self.inputs[id as usize] = Some(device);
    }

    pub(crate) fn add_output_device(&mut self, id: u8, device: Box<OutputDevice>) {
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
        match (self.registers.get(&RegisterType::H).unwrap(), self.registers.get(&RegisterType::L).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                two_bytes_to_word(*h_value, *l_value),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    pub(crate) fn get_current_bc_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::B).unwrap(), self.registers.get(&RegisterType::C).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                two_bytes_to_word(*h_value, *l_value),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    pub(crate) fn get_current_de_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::D).unwrap(), self.registers.get(&RegisterType::E).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                two_bytes_to_word(*h_value, *l_value),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    pub(crate) fn get_value_in_memory_at_hl(&self) -> u8 {
        let source_value_address: u16 = self.get_current_hl_value();
        self.memory[source_value_address as usize].get()
    }

    #[inline]
    pub(crate) fn set_value_in_memory_at_hl(&mut self, value: u8) {
        let source_value_address: u16 = self.get_current_hl_value();
        self.memory[source_value_address as usize].set(value);
    }

    #[inline]
    pub(crate) fn get_current_a_value(&self) -> u8 {
        self.get_current_single_register_value(&RegisterType::A)
    }

    #[inline]
    pub(crate) fn get_current_sp_value(&self) -> u16 {
        match self.registers.get(&RegisterType::Sp).unwrap() {
            Register::DoubleRegister { value } => *value,
            _ => panic!("SP register wasn't a word!")
        }
    }

    #[inline]
    pub(crate) fn get_current_single_register_value(&self, register: &RegisterType) -> u8 {
        if let Register::SingleRegister { value } = self.registers.get(register).unwrap() {
            *value
        } else {
            panic!("{} register is double. Can't happen.", register.to_string())
        }
    }

    #[inline]
    pub(crate) fn save_to_a(&mut self, new_value: u8) {
        self.save_to_single_register(new_value, &RegisterType::A)
    }

    #[inline]
    pub(crate) fn save_to_double_register(&mut self, new_value: u16, register: &RegisterType) {
        if let Some(Register::DoubleRegister { value }) = self.registers.get_mut(register) {
            *value = new_value;
        }
    }

    #[inline]
    pub(crate) fn save_to_single_register(&mut self, new_value: u8, register: &RegisterType) {
        if let Some(Register::SingleRegister { value }) = self.registers.get_mut(register) {
            *value = new_value;
        }
    }

    #[inline]
    pub(crate) fn perform_jump(&mut self, high_byte: u8, low_byte: u8) {
        let new_pc = two_bytes_to_word(high_byte, low_byte);
        self.pc = new_pc;
    }
}
