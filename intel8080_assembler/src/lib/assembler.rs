extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Instruction, Intel8080Instruction, Location, RegisterType, ROM_MEMORY_LIMIT};
use std::collections::HashMap;
use super::{Expression, Label};

pub struct Assembler {
    bytes: HashMap<Label, u8>,
    labels: HashMap<Label, u16>,
    pc: u16,
    rom: [u8; ROM_MEMORY_LIMIT],
    words: HashMap<Label, u16>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            bytes: HashMap::new(),
            labels: HashMap::new(),
            pc: 0,
            rom: [0; ROM_MEMORY_LIMIT],
            words: HashMap::new(),
        }
    }

    pub fn assemble(mut self, expressions: Vec<Expression>) -> Result<[u8; ROM_MEMORY_LIMIT], Error> {
        for expression in expressions {
            match expression {
                Expression::LabelDefinition(label) => { self.labels.insert(label, self.pc); },
                Expression::ByteDefinition { label, value } => { self.bytes.insert(label, value); },
                Expression::WordDefinition { label, value } => { self.words.insert(label, value); },
                Expression::Instruction(instruction) => {
                    self.pc += instruction.size()? as u16;
                    self.add_instruction(instruction);
                }
            }
        }
        Ok(self.rom)
    }

    fn add_instruction(&mut self, instruction: Intel8080Instruction) {
        for byte in self.bytes_for_instruction(instruction) {
            self.rom[self.pc as usize] = byte;
            self.pc += 1;
        }
    }

    fn bytes_for_instruction(&self, instruction: Intel8080Instruction) -> Vec<u8> {
        let mut res = Vec::with_capacity(3);
        match instruction {
            Intel8080Instruction::Noop => res.push(0x00),
            Intel8080Instruction::Lxi { register: RegisterType::B, low_byte, high_byte } => {
                res.push(0x01);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Stax { register: RegisterType::B } => res.push(0x02),
            Intel8080Instruction::Inx { register: RegisterType::B } => res.push(0x03),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::B } } =>
                res.push(0x04),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::B } } =>
                res.push(0x05),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::B },
                byte
            } => {
                res.push(0x06);
                res.push(byte);
            },
            Intel8080Instruction::Rlc => res.push(0x07),
            Intel8080Instruction::Dad { register: RegisterType::B } => res.push(0x09),
            Intel8080Instruction::Ldax { register: RegisterType::B } => res.push(0x0a),
            Intel8080Instruction::Dcx { register: RegisterType::B } => res.push(0x0b),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::C } } =>
                res.push(0x0c),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::C } } =>
                res.push(0x0d),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::C },
                byte
            } => {
                res.push(0x0e);
                res.push(byte);
            },
            Intel8080Instruction::Rrc => res.push(0x0f),
            Intel8080Instruction::Lxi { register: RegisterType::D, low_byte, high_byte } => {
                res.push(0x11);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Stax { register: RegisterType::D } => res.push(0x12),
            Intel8080Instruction::Inx { register: RegisterType::D } => res.push(0x13),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::D } } =>
                res.push(0x14),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::D } } =>
                res.push(0x15),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::D },
                byte
            } => {
                res.push(0x16);
                res.push(byte);
            },
            Intel8080Instruction::Ral => res.push(0x17),
            Intel8080Instruction::Dad { register: RegisterType::D } => res.push(0x19),
            Intel8080Instruction::Ldax { register: RegisterType::D } => res.push(0x1a),
            Intel8080Instruction::Dcx { register: RegisterType::D } => res.push(0x1b),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::E } } =>
                res.push(0x1c),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::E } } =>
                res.push(0x1d),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::E },
                byte
            } => {
                res.push(0x1e);
                res.push(byte);
            },
            Intel8080Instruction::Rar => res.push(0x1f),
            Intel8080Instruction::Lxi { register: RegisterType::H, low_byte, high_byte } => {
                res.push(0x21);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Shld { address: [low_byte, high_byte] } => {
                res.push(0x22);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Inx { register: RegisterType::H } => res.push(0x23),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::H } } =>
                res.push(0x24),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::H } } =>
                res.push(0x25),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::H },
                byte
            } => {
                res.push(0x26);
                res.push(byte);
            },
            Intel8080Instruction::Daa => res.push(0x27),
            Intel8080Instruction::Dad { register: RegisterType::H } => res.push(0x29),
            Intel8080Instruction::Lhld { address: [low_byte, high_byte] } => {
                res.push(0x2a);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Dcx { register: RegisterType::H } => res.push(0x2b),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::L } } =>
                res.push(0x2c),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::L } } =>
                res.push(0x2d),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::L },
                byte
            } => {
                res.push(0x2e);
                res.push(byte);
            },
            Intel8080Instruction::Cma => res.push(0x2f),
            Intel8080Instruction::Lxi { register: RegisterType::Sp, low_byte, high_byte } => {
                res.push(0x31);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Sta { address: [low_byte, high_byte] } => {
                res.push(0x32);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Inx { register: RegisterType::Sp } => res.push(0x33),
            Intel8080Instruction::Inr { source: Location::Memory } => res.push(0x34),
            Intel8080Instruction::Dcr { source: Location::Memory } => res.push(0x35),
            Intel8080Instruction::Mvi { source: Location::Memory, byte } => {
                res.push(0x36);
                res.push(byte);
            },
            Intel8080Instruction::Stc => res.push(0x37),
            Intel8080Instruction::Dad { register: RegisterType::Sp } => res.push(0x39),
            Intel8080Instruction::Lda { address: [low_byte, high_byte] } => {
                res.push(0x3a);
                res.push(low_byte);
                res.push(high_byte);
            },
            Intel8080Instruction::Dcx { register: RegisterType::Sp } => res.push(0x3b),
            Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::A } } =>
                res.push(0x3c),
            Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::A } } =>
                res.push(0x3d),
            Intel8080Instruction::Mvi {
                source: Location::Register { register: RegisterType::A },
                byte
            } => {
                res.push(0x3e);
                res.push(byte);
            },
            Intel8080Instruction::Cmc => res.push(0x3f),
            _ => panic!("unfined method"),
        }
        res
    }
}