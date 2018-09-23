extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Instruction, Intel8080Instruction, ROM_MEMORY_LIMIT};
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
                Expression::LabelDefinition(label) => self.labels.insert(label, self.pc),
                Expression::ByteDefinition(label, byte) => self.bytes.insert(label, byte),
                Expression::WordDefinition(label, word) => self.words.insert(label, word),
                Expression::Instruction(instruction) => {
                    self.pc += instruction.size()? as u16;
                    self.add_instruction(instruction);
                }
            }
        }
        Ok(self.rom)
    }

    fn add_instruction(&mut self, instruction: Intel8080Instruction) {

    }
}