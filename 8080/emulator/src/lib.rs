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
            Instruction::Adc { source: Destiny::Register { register } } => self.execute_adc_by_register(&register),
            Instruction::Adc { source: Destiny::Memory } => self.execute_adc_by_memory(),
            Instruction::Add { source: Destiny::Register { register } } => self.execute_add_by_register(&register),
            Instruction::Add { source: Destiny::Memory } => self.execute_add_by_memory(),
            Instruction::Adi { byte } => self.execute_adi(byte),
            Instruction::Ana { source: Destiny::Register { register } } => self.execute_ana_by_register(&register),
            Instruction::Ana { source: Destiny::Memory } => self.execute_ana_by_memory(),
            Instruction::Cma => self.execute_cma(),
            Instruction::Cmc => self.execute_cmc(),
            Instruction::Daa => self.execute_daa(),
            Instruction::Dcr { source: Destiny::Register { register } } => self.execute_dcr_by_register(&register),
            Instruction::Dcr { source: Destiny::Memory } => self.execute_dcr_by_memory(),
            Instruction::Inr { source: Destiny::Register { register } } => self.execute_inr_by_register(&register),
            Instruction::Inr { source: Destiny::Memory } => self.execute_inr_by_memory(),
            Instruction::Ldax { register } => self.execute_ldax(&register),
            Instruction::Mov { destiny, source } => self.execute_mov(&destiny, &source),
            Instruction::Stax { register } => self.execute_stax(&register),
            Instruction::Stc => self.execute_stc(),
            Instruction::Sbb { source: Destiny::Register { register } } => self.execute_sbb_by_register(&register),
            Instruction::Sbb { source: Destiny::Memory } => self.execute_sbb_by_memory(),
            Instruction::Sub { source: Destiny::Register { register } } => self.execute_sub_by_register(&register),
            Instruction::Sub { source: Destiny::Memory } => self.execute_sub_by_memory(),
            Instruction::Xra { source: Destiny::Register { register } } => self.execute_xra_by_register(&register),
            Instruction::Xra { source: Destiny::Memory } => self.execute_xra_by_memory(),
            _ => println!("Execute: {}", instruction.to_string()),
        }
    }

    pub fn is_done(&self) -> bool {
        (self.pc as usize) >= ROM_MEMORY_LIMIT
    }

    fn execute_adi(&mut self, byte: u8) {
        let destiny_value = self.get_current_a_value() as u16;
        let new_value = self.perform_add_with_carry(byte as u16, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_adc_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    fn execute_adc_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let carry_as_u16 = self.flags.carry as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        let new_value = self.perform_add_with_carry(carry_as_u16, new_value as u16);
        self.save_to_a(new_value);
    }

    fn execute_add_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_add_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_add_with_carry(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_ana_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_and(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_ana_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_and(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    #[inline]
    fn execute_cma(&mut self) {
        let destiny_value = self.get_current_a_value();
        self.save_to_a(!destiny_value);
    }

    #[inline]
    fn execute_cmc(&mut self) {
        self.flags.carry = !self.flags.carry;
    }

    fn execute_daa(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let least_significant = destiny_value & 0x0f;
        let mut result = destiny_value;
        if least_significant > 9 || self.flags.auxiliary_carry {
            result += 6;
            self.flags.auxiliary_carry = (result & 0x0f) < least_significant;
        }
        let most_significant = (result & 0xf0) >> 4;
        if most_significant > 9 || self.flags.carry {
            result = result | ((most_significant + 6) << 4);
            if result > 0xff {
                self.flags.carry = true;
            }
        }
        self.update_flags(result, false);
        self.save_to_a(result as u8);
    }

    fn execute_dcr_by_register(&mut self, register_type: &RegisterType) {
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_sub_without_carry(source_value, 1);
        self.save_to_single_register(new_value, register_type);
    }

    fn execute_dcr_by_memory(&mut self) {
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_sub_without_carry(source_value, 1);
        self.set_value_in_memory_at_hl(new_value);
    }

    fn execute_inr_by_register(&mut self, register_type: &RegisterType) {
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_add_without_carry(source_value, 1);
        self.save_to_single_register(new_value, register_type);
    }

    fn execute_inr_by_memory(&mut self) {
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_add_without_carry(source_value, 1);
        self.set_value_in_memory_at_hl(new_value);
    }

    fn execute_ldax(&mut self, register: &RegisterType) {
        let source_address = match register {
            RegisterType::B => self.get_current_bc_value(),
            RegisterType::D => self.get_current_de_value(),
            _ => panic!("Register {} is not a valid input of STAX", register.to_string()),
        } as usize;
        let value = self.memory[source_address];
        self.save_to_a(value);
    }

    #[inline]
    fn execute_mov(&mut self, destiny: &RegisterType, source: &RegisterType) {
        let source_value = self.get_current_single_register_value(source);
        self.save_to_single_register(source_value, destiny);
    }

    fn execute_stax(&mut self, register: &RegisterType) {
        let value = self.get_current_a_value();
        let destiny_address = match register {
            RegisterType::B => self.get_current_bc_value(),
            RegisterType::D => self.get_current_de_value(),
            _ => panic!("Register {} is not a valid input of STAX", register.to_string()),
        } as usize;
        self.memory[destiny_address] = value;
    }

    #[inline]
    fn execute_stc(&mut self) {
        self.flags.carry = true;
    }

    fn execute_sbb_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let carry = self.flags.carry as u8;
        let source_value = (self.get_current_single_register_value(register_type) + carry) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    fn execute_sbb_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let carry = self.flags.carry as u8;
        let source_value = (self.get_value_in_memory_at_hl() + carry) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    fn execute_sub_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    fn execute_sub_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        let new_value = self.perform_sub_with_carry(destiny_value, source_value);
        self.save_to_a(new_value);
    }

    fn execute_xra_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_xor(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_xra_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_xor(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    #[inline]
    fn get_current_hl_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::H).unwrap(), self.registers.get(&RegisterType::L).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    fn get_current_bc_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::B).unwrap(), self.registers.get(&RegisterType::C).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    fn get_current_de_value(&self) -> u16 {
        match (self.registers.get(&RegisterType::D).unwrap(), self.registers.get(&RegisterType::E).unwrap()) {
            (Register::SingleRegister { value: h_value }, Register::SingleRegister { value: l_value }) =>
                ((*h_value as u16) << 8) | (*l_value as u16),
            _ => panic!("Register HL either not registered or Double. Can't happen!"),
        }
    }

    #[inline]
    fn get_value_in_memory_at_hl(&self) -> u8 {
        let source_value_address: u16 = self.get_current_hl_value();
        self.memory[source_value_address as usize]
    }

    #[inline]
    fn set_value_in_memory_at_hl(&mut self, value: u8) {
        let source_value_address: u16 = self.get_current_hl_value();
        self.memory[source_value_address as usize] = value;
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
        self.save_to_single_register(new_value, &RegisterType::A)
    }

    #[inline]
    fn save_to_single_register(&mut self, new_value: u8, register: &RegisterType) {
        if let Some(Register::SingleRegister { value }) = self.registers.get_mut(register) {
            *value = new_value;
        }
    }

    #[inline]
    fn perform_add_with_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_add(destiny, source, true)
    }

    #[inline]
    fn perform_add_without_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_add(destiny, source, true)
    }

    #[inline]
    fn perform_add(&mut self, destiny: u16, source: u16, with_carry: bool) -> u8 {
        let answer: u16 = source + destiny;
        self.update_flags(answer, with_carry);
        self.update_auxiliar_carry(destiny, source);
        (answer & 0xff) as u8
    }

    #[inline]
    fn perform_and(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny & source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        answer
    }

    #[inline]
    fn perform_sub_with_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_sub(destiny, source, true)
    }

    #[inline]
    fn perform_sub_without_carry(&mut self, destiny: u16, source: u16) -> u8 {
        self.perform_sub(destiny, source, true)
    }

    #[inline]
    fn perform_sub(&mut self, destiny: u16, source: u16, with_carry: bool) -> u8 {
        let answer: u16 = destiny + !source + 1;
        self.update_flags(answer, false);
        if with_carry {
            self.flags.carry = answer <= 0xff;
        }
        self.update_auxiliar_carry_with_sub(destiny, source);
        (answer & 0xff) as u8
    }
    #[inline]
    fn perform_xor(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny ^ source;
        self.update_flags(answer as u16, false);
        self.flags.carry = false;
        answer
    }

    #[inline]
    fn update_flags(&mut self, answer: u16, with_carry: bool) {
        self.flags.zero = (answer & 0xff) == 0;
        self.flags.sign = (answer & 0x80) != 0;
        if with_carry {
            self.flags.carry = answer > 0xff;
        }
        self.flags.parity = (answer & 0xff) % 2 == 0;
    }

    #[inline]
    fn update_auxiliar_carry_with_sub(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (!source & 0x0f) + 1 > 0x0f;
    }

    #[inline]
    fn update_auxiliar_carry(&mut self, destiny: u16, source: u16) {
        self.flags.auxiliary_carry = (destiny & 0x0f) + (source & 0x0f) > 0x0f;
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.pc as usize;
        let to = min(from+3, self.memory.len());
        &(self.memory[from..to])
    }
}
