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
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x40),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x41),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x42),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x43),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x44),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x45),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Memory,
            } => res.push(0x46),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::B },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x47),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x48),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x49),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x4a),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x4b),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x4c),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x4d),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Memory,
            } => res.push(0x4e),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::C },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x4f),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x50),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x51),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x52),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x53),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x54),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x55),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Memory,
            } => res.push(0x56),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::D },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x57),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x58),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x59),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x5a),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x5b),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x5c),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x5d),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Memory,
            } => res.push(0x5e),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::E },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x5f),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x60),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x61),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x62),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x63),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x64),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x65),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Memory,
            } => res.push(0x66),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::H },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x67),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x68),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x69),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x6a),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x6b),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x6c),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x6d),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Memory,
            } => res.push(0x6e),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::L },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x6f),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x70),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x71),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x72),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x73),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x74),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x75),
            Intel8080Instruction::Hlt => res.push(0x76),
            Intel8080Instruction::Mov {
                destiny: Location::Memory,
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x77),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::B }
            } => res.push(0x78),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::C }
            } => res.push(0x79),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::D }
            } => res.push(0x7a),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::E }
            } => res.push(0x7b),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::H }
            } => res.push(0x7c),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::L }
            } => res.push(0x7d),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Memory,
            } => res.push(0x7e),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: RegisterType::A },
                source: Location::Register { register: RegisterType::A }
            } => res.push(0x7f),
            _ => panic!("unfined method"),
        }
        res
    }
}