extern crate disassembler_8080;

use disassembler_8080::{Destiny, Instruction, RegisterType};
use std::cmp::min;
use std::collections::HashMap;

const ROM_MEMORY_LIMIT: usize = 8192;

enum Register {
    SingleRegister { value: u8 },
    DoubleRegister { value: u16 },
}

impl Register {
    fn new(t: RegisterType) -> Register {
        match t {
            RegisterType::Sp | RegisterType::M | RegisterType::Psw =>
                Register::DoubleRegister { value: 0 },
            _ => Register::SingleRegister { value: 0 },
        }
    }
}

struct Flags {
    sign: bool,
    zero: bool,
    parity: bool,
    carry: bool,
    auxiliary_carry: bool,
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
    registers: HashMap<RegisterType, Register>,
    pc: u16,
    memory: [u8; ROM_MEMORY_LIMIT * 2],
    flags: Flags,
    interruptions_enabled: bool,
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

    pub fn execute(&mut self) {
        let instruction = Instruction::from_bytes(self.get_next_instruction_bytes());
        self.pc += instruction.size() as u16;
        match instruction {
            Instruction::Adc { source: Destiny::RegisterDestiny { register } } => self.execute_adc_by_register(&register),
            Instruction::Adc { source: Destiny::MemoryDestiny } => self.execute_adc_by_memory(),
            Instruction::Add { source: Destiny::RegisterDestiny { register } } => self.execute_add_by_register(&register),
            Instruction::Add { source: Destiny::MemoryDestiny } => self.execute_add_by_memory(),
            Instruction::Adi { byte } => self.execute_adi(byte),
            _ => println!("Execute: {}", instruction.to_string()),
        }
    }

    pub fn is_done(&self) -> bool {
        (self.pc as usize) >= ROM_MEMORY_LIMIT
    }

    fn execute_adi(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        let new_value = self.perform_add(byte as u16, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_adc_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add(source_value, destiny_value);
        let new_value = self.perform_add(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    fn execute_adc_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value_address: u16 = self.get_current_hl_value();
        let source_value = self.memory[source_value_address as usize] as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add(source_value, destiny_value);
        let new_value = self.perform_add(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    fn execute_add_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_add(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_add_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value_address: u16 = self.get_current_hl_value();
        let source_value = self.memory[source_value_address as usize] as u16;
        let new_value = self.perform_add(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    #[inline]
    fn get_current_hl_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::H).unwrap(), self.registers.get(&RegisterType::H).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    fn get_current_a_value(&self) -> u8 {
        self.get_current_single_register_value(&RegisterType::A)
    }

    #[inline]
    fn get_current_single_register_value(&self, r: &RegisterType) -> u8 {
        if let Register::SingleRegister { value } = self.registers.get(r).unwrap() {
            *value
        } else {
            panic!("{} register is double. Can't happen.", r.to_string())
        }
    }

    #[inline]
    fn save_to_a(&mut self, new_value: u8) {
        if let Some(Register::SingleRegister { value }) = self.registers.get_mut(&RegisterType::A) {
            *value = new_value;
        }
    }

    #[inline]
    fn perform_add(&mut self, destiny: u16, source: u16) -> u8 {
        let answer: u16 = source + destiny;
        self.flags.zero = (answer & 0xff) == 0;
        self.flags.sign = (answer & 0x80) != 0;
        self.flags.carry = answer > 0xff;
        self.flags.parity = (answer & 0xff) % 2 == 0;
        self.flags.auxiliary_carry = (source & 0x0f) + (destiny & 0x0f) > 0x0f;
        (destiny & 0xff) as u8
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.pc as usize;
        let to = min(from+3, self.memory.len());
        &(self.memory[from..to])
    }
}
