extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Instruction, Intel8080Instruction, RegisterType, ROM_MEMORY_LIMIT};
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
        let mut res = Vec::new();
        match instruction {
            Intel8080Instruction::Noop => res.push(0x00),
            Intel8080Instruction::Lxi { register: RegisterType::B, low_byte, high_byte } => {
                res.push(0x01);
                res.push(low_byte);
                res.push(high_byte);
            },
            _ => panic!("unfined method"),
        }
        res
    }
}