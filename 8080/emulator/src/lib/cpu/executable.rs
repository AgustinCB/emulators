extern crate disassembler_8080;

use self::disassembler_8080::{Instruction, Location, RegisterType};
use cpu::state::{Register, State};
use std::cmp::min;

pub trait Executable {
    fn execute(&mut self);
    fn execute_instruction(&mut self, instruction: Instruction);
}

impl State {
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

    #[inline]
    fn execute_cmp_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_current_single_register_value(register_type) as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
    }

    #[inline]
    fn execute_cmp_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value() as u16;
        let source_value = self.get_value_in_memory_at_hl() as u16;
        self.perform_sub_with_carry(destiny_value, source_value);
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

    fn execute_dad(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_hl_value() as u32;
        let source_value = match register_type {
            RegisterType::B => self.get_current_bc_value() as u32,
            RegisterType::D => self.get_current_de_value() as u32,
            RegisterType::H => self.get_current_hl_value() as u32,
            RegisterType::Sp => self.get_current_sp_value() as u32,
            _ => panic!("{} is not a valid DAD argument!", register_type.to_string()),
        };
        let result = destiny_value + source_value;
        self.flags.carry = result > 0xffff;
        self.save_to_single_register((result >> 8) as u8, &RegisterType::H);
        self.save_to_single_register(result as u8, &RegisterType::L);
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

    fn execute_dcx(&mut self, register_type: &RegisterType) {
        self.perform_step_on_double_register(register_type, false);
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

    fn execute_inx(&mut self, register_type: &RegisterType) {
        self.perform_step_on_double_register(register_type, true);
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

    fn execute_lxi(&mut self, register_type: &RegisterType, high_byte: u8, low_byte: u8) {
        match register_type {
            RegisterType::B => {
                self.save_to_single_register(high_byte, &RegisterType::B);
                self.save_to_single_register(low_byte, &RegisterType::C);
            },
            RegisterType::D => {
                self.save_to_single_register(high_byte, &RegisterType::D);
                self.save_to_single_register(low_byte, &RegisterType::E);
            },
            RegisterType::H => {
                self.save_to_single_register(high_byte, &RegisterType::H);
                self.save_to_single_register(low_byte, &RegisterType::L);
            },
            RegisterType::Sp =>
                self.save_to_double_register((high_byte as u16) << 8 | (low_byte as u16), &RegisterType::Sp),
            _ => panic!("Register {} is not a valid input of LXI", register_type.to_string()),
        }
    }

    #[inline]
    fn execute_mov(&mut self, destiny: &Location, source: &Location) {
        match (destiny, source) {
            (Location::Register { register: destiny }, Location::Register { register: source }) =>
                self.execute_mov_register_to_register(&destiny, &source),
            (Location::Register { register: destiny }, Location::Memory) =>
                self.execute_mov_memory_to_register(&destiny),
            (Location::Memory, Location::Register { register: source }) =>
                self.execute_mov_register_to_memory(&source),
            (Location::Memory, Location::Memory) =>
                panic!("MOV (HL),(HL) can't happen!")
        }
    }

    #[inline]
    fn execute_mov_register_to_register(&mut self, destiny: &RegisterType, source: &RegisterType) {
        let source_value = self.get_current_single_register_value(source);
        self.save_to_single_register(source_value, destiny);
    }

    #[inline]
    fn execute_mov_memory_to_register(&mut self, destiny: &RegisterType) {
        let source_value = self.get_value_in_memory_at_hl();
        self.save_to_single_register(source_value, destiny);
    }

    #[inline]
    fn execute_mov_register_to_memory(&mut self, source: &RegisterType) {
        let source_value = self.get_current_single_register_value(source);
        self.set_value_in_memory_at_hl(source_value);
    }

    fn execute_ora_by_register(&mut self, register_type: &RegisterType) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_current_single_register_value(register_type);
        let new_value = self.perform_or(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_ora_by_memory(&mut self) {
        let destiny_value = self.get_current_a_value();
        let source_value = self.get_value_in_memory_at_hl();
        let new_value = self.perform_or(source_value, destiny_value);
        self.save_to_a(new_value);
    }

    fn execute_push(&mut self, register: &RegisterType) {
        let sp = self.get_current_sp_value() as usize;
        let (first_byte, second_byte) = match register {
            RegisterType::B =>
                (self.get_current_single_register_value(&RegisterType::B), self.get_current_single_register_value(&RegisterType::C)),
            RegisterType::D =>
                (self.get_current_single_register_value(&RegisterType::D), self.get_current_single_register_value(&RegisterType::E)),
            RegisterType::H =>
                (self.get_current_single_register_value(&RegisterType::H), self.get_current_single_register_value(&RegisterType::L)),
            RegisterType::Psw =>
                (self.get_current_a_value(), self.get_current_flags_byte()),
            _ => panic!("{} is not a valid register for push!", register.to_string()),
        };
        self.memory[sp-1] = first_byte;
        self.memory[sp-2] = second_byte;
        self.save_to_double_register((sp-2) as u16, &RegisterType::Sp);
    }

    fn execute_pop(&mut self, register: &RegisterType) {
        let sp = self.get_current_sp_value() as usize;
        let first_byte = self.memory[sp+1];
        let second_byte = self.memory[sp];
        match register {
            RegisterType::B => {
                self.save_to_single_register(first_byte, &RegisterType::B);
                self.save_to_single_register(first_byte, &RegisterType::C);
            },
            RegisterType::D => {
                self.save_to_single_register(first_byte, &RegisterType::D);
                self.save_to_single_register(first_byte, &RegisterType::E);
            },
            RegisterType::H => {
                self.save_to_single_register(first_byte, &RegisterType::H);
                self.save_to_single_register(first_byte, &RegisterType::L);
            },
            RegisterType::Psw => {
                self.save_to_a(first_byte);
                self.set_flags_byte(second_byte);
            },
            _ => panic!("{} is not a valid register for push!", register.to_string()),
        };
        self.save_to_double_register((sp+2) as u16, &RegisterType::Sp);
    }

    #[inline]
    fn execute_ral(&mut self) {
        let value = self.get_current_a_value().rotate_left(1);
        let carry_mask = if self.flags.carry {
            0x80
        } else {
            0
        };
        self.flags.carry = (value & 0x01) != 0;
        self.save_to_a(value | carry_mask);
    }

    #[inline]
    fn execute_rar(&mut self) {
        let value = self.get_current_a_value().rotate_right(1);
        let carry_mask = if self.flags.carry {
            0x80
        } else {
            0
        };
        self.flags.carry = (value & 0x80) != 0;
        self.save_to_a(value | carry_mask);
    }

    #[inline]
    fn execute_rlc(&mut self) {
        let value = self.get_current_a_value().rotate_left(1);
        self.flags.carry = (value & 0x01) != 0;
        self.save_to_a(value);
    }

    #[inline]
    fn execute_rrc(&mut self) {
        let value = self.get_current_a_value().rotate_right(1);
        self.flags.carry = (value & 0x80) != 0;
        self.save_to_a(value);
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

    fn execute_sphl(&mut self) {
        let hl = self.get_current_hl_value();
        self.save_to_double_register(hl, &RegisterType::Sp);
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

    fn execute_xchg(&mut self) {
        let d_value = self.get_current_single_register_value(&RegisterType::D);
        let e_value = self.get_current_single_register_value(&RegisterType::E);
        let h_value = self.get_current_single_register_value(&RegisterType::H);
        let l_value = self.get_current_single_register_value(&RegisterType::L);
        self.save_to_single_register(h_value, &RegisterType::D);
        self.save_to_single_register(l_value, &RegisterType::E);
        self.save_to_single_register(d_value, &RegisterType::H);
        self.save_to_single_register(e_value, &RegisterType::L);
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

    fn execute_xthl(&mut self) {
        let sp = self.get_current_sp_value() as usize;
        let first_byte = self.memory[sp+1];
        let second_byte = self.memory[sp];
        let h_value = self.get_current_single_register_value(&RegisterType::H);
        let l_value = self.get_current_single_register_value(&RegisterType::L);
        self.save_to_single_register(first_byte, &RegisterType::H);
        self.save_to_single_register(second_byte, &RegisterType::L);
        self.memory[sp+1] = h_value;
        self.memory[sp] = l_value;
    }

    #[inline]
    fn get_current_flags_byte(&self) -> u8 {
        (self.flags.zero as u8) |
            (self.flags.sign as u8) << 1 |
            (self.flags.parity as u8) << 2 |
            (self.flags.carry as u8) << 3 |
            (self.flags.auxiliary_carry as u8) << 4
    }

    #[inline]
    fn set_flags_byte(&mut self, byte: u8) {
        self.flags.zero = (byte & 0x01) == 0x01;
        self.flags.sign = (byte & 0x02) == 0x02;
        self.flags.parity = (byte & 0x04) == 0x04;
        self.flags.carry = (byte & 0x08) == 0x08;
        self.flags.auxiliary_carry = (byte & 0x10) == 0x10;
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
    fn get_current_sp_value(&self) -> u16 {
        match self.registers.get(&RegisterType::Sp).unwrap() {
            Register::DoubleRegister { value } => *value,
            _ => panic!("SP register wasn't a word!")
        }
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
    fn save_to_double_register(&mut self, new_value: u16, register: &RegisterType) {
        if let Some(Register::DoubleRegister { value }) = self.registers.get_mut(register) {
            *value = new_value;
        }
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
    fn perform_step_on_double_register(&mut self, register_type: &RegisterType, inc: bool) {
        let destiny_value = match register_type {
            RegisterType::B => self.get_current_bc_value() as u32,
            RegisterType::D => self.get_current_de_value() as u32,
            RegisterType::H => self.get_current_hl_value() as u32,
            RegisterType::Sp => self.get_current_sp_value() as u32,
            _ => panic!("{} is not a valid INX argument!", register_type.to_string()),
        };
        let result = if inc { destiny_value+1 } else { destiny_value - 1 };
        match register_type {
            RegisterType::B => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::B);
                self.save_to_single_register(result as u8, &RegisterType::C);
            } 
            RegisterType::D => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::D);
                self.save_to_single_register(result as u8, &RegisterType::E);
            }
            RegisterType::H => {
                self.save_to_single_register((result >> 8) as u8, &RegisterType::H);
                self.save_to_single_register(result as u8, &RegisterType::L);
            }
            RegisterType::Sp => self.save_to_double_register(result as u16, &RegisterType::Sp),
            _ => panic!("{} is not a valid INX argument!", register_type.to_string()),
        }
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
    fn perform_or(&mut self, destiny: u8, source: u8) -> u8 {
        let answer = destiny | source;
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

impl Executable for State {
    fn execute(&mut self) {
        let instruction = Instruction::from_bytes(self.get_next_instruction_bytes());
        self.pc += instruction.size() as u16;
        self.execute_instruction(instruction);
    }

    fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Adc { source: Location::Register { register } } => self.execute_adc_by_register(&register),
            Instruction::Adc { source: Location::Memory } => self.execute_adc_by_memory(),
            Instruction::Add { source: Location::Register { register } } => self.execute_add_by_register(&register),
            Instruction::Add { source: Location::Memory } => self.execute_add_by_memory(),
            Instruction::Adi { byte } => self.execute_adi(byte),
            Instruction::Ana { source: Location::Register { register } } => self.execute_ana_by_register(&register),
            Instruction::Ana { source: Location::Memory } => self.execute_ana_by_memory(),
            Instruction::Cma => self.execute_cma(),
            Instruction::Cmc => self.execute_cmc(),
            Instruction::Cmp { source: Location::Register { register } } => self.execute_cmp_by_register(&register),
            Instruction::Cmp { source: Location::Memory } => self.execute_cmp_by_memory(),
            Instruction::Daa => self.execute_daa(),
            Instruction::Dad { register } => self.execute_dad(&register),
            Instruction::Dcr { source: Location::Register { register } } => self.execute_dcr_by_register(&register),
            Instruction::Dcr { source: Location::Memory } => self.execute_dcr_by_memory(),
            Instruction::Dcx { register } => self.execute_dcx(&register),
            Instruction::Inr { source: Location::Register { register } } => self.execute_inr_by_register(&register),
            Instruction::Inr { source: Location::Memory } => self.execute_inr_by_memory(),
            Instruction::Inx { register } => self.execute_inx(&register),
            Instruction::Ldax { register } => self.execute_ldax(&register),
            Instruction::Lxi { register, low_byte, high_byte } => self.execute_lxi(&register, high_byte, low_byte),
            Instruction::Mov { destiny, source } => self.execute_mov(&destiny, &source),
            Instruction::Pop { register } => self.execute_pop(&register),
            Instruction::Push { register } => self.execute_push(&register),
            Instruction::Ora { source: Location::Register { register } } => self.execute_ora_by_register(&register),
            Instruction::Ora { source: Location::Memory } => self.execute_ora_by_memory(),
            Instruction::Ral => self.execute_ral(),
            Instruction::Rar => self.execute_rar(),
            Instruction::Rlc => self.execute_rlc(),
            Instruction::Rrc => self.execute_rrc(),
            Instruction::Sbb { source: Location::Register { register } } => self.execute_sbb_by_register(&register),
            Instruction::Sbb { source: Location::Memory } => self.execute_sbb_by_memory(),
            Instruction::Stax { register } => self.execute_stax(&register),
            Instruction::Stc => self.execute_stc(),
            Instruction::Sphl => self.execute_sphl(),
            Instruction::Sub { source: Location::Register { register } } => self.execute_sub_by_register(&register),
            Instruction::Sub { source: Location::Memory } => self.execute_sub_by_memory(),
            Instruction::Xchg => self.execute_xchg(),
            Instruction::Xra { source: Location::Register { register } } => self.execute_xra_by_register(&register),
            Instruction::Xra { source: Location::Memory } => self.execute_xra_by_memory(),
            Instruction::Xthl => self.execute_xthl(),
            _ => println!("Execute: {}", instruction.to_string()),
        }
    }
}
