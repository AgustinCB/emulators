extern crate disassembler_8080;

use self::disassembler_8080::RegisterType;
use std::collections::HashMap;

pub(crate) const ROM_MEMORY_LIMIT: usize = 8192;

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

pub struct Cpu {
    pub(crate) registers: HashMap<RegisterType, Register>,
    pub(crate) pc: u16,
    pub(crate) memory: [u8; ROM_MEMORY_LIMIT * 8],
    pub(crate) flags: Flags,
    pub(crate) interruptions_enabled: bool,
}

impl Cpu {
    pub fn new(rom_memory: [u8; ROM_MEMORY_LIMIT]) -> Cpu {
        let mut registers = HashMap::new();
        let mut memory = [0; ROM_MEMORY_LIMIT * 8];
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
        Cpu {
            registers,
            pc: 0,
            memory,
            flags: Flags::new(),
            interruptions_enabled: true,
        }
    }

    pub fn is_done(&self) -> bool {
        (self.pc as usize) >= ROM_MEMORY_LIMIT
    }

    #[inline]
    pub(crate) fn update_flags(&mut self, answer: u16, with_carry: bool) {
        self.flags.zero = (answer & 0xff) == 0;
        self.flags.sign = (answer & 0x80) != 0;
        if with_carry {
            self.flags.carry = answer > 0xff;
        }
        self.flags.parity = (answer & 0xff) % 2 == 0;
    }

    #[inline]
    pub(crate) fn update_auxiliary_carry_with_sub(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (!source & 0x0f) + 1 > 0x0f;
    }

    #[inline]
    pub(crate) fn update_auxiliary_carry(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (source & 0x0f) > 0x0f;
    }

    #[inline]
    pub(crate) fn get_current_hl_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::H).unwrap(), self.registers.get(&RegisterType::L).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    pub(crate) fn get_current_bc_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::B).unwrap(), self.registers.get(&RegisterType::C).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    pub(crate) fn get_current_de_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::D).unwrap(), self.registers.get(&RegisterType::E).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
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
    pub(crate) fn get_current_single_register_value(&self, r: &RegisterType) -> u8 {
        if let Register::SingleRegister { value } = self.registers.get(r).unwrap() {
            *value
        } else {
            panic!("{} register is double. Can't happen.", r.to_string())
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
}
