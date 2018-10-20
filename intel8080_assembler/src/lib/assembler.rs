extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Location, RegisterType, ROM_MEMORY_LIMIT};
use std::collections::HashMap;
use super::*;

pub struct Assembler {
    words: HashMap<LabelExpression, u8>,
    labels: HashMap<LabelExpression, u16>,
    pc: u16,
    rom: [u8; ROM_MEMORY_LIMIT],
    two_words: HashMap<LabelExpression, u16>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            words: HashMap::new(),
            labels: HashMap::new(),
            pc: 0,
            rom: [0; ROM_MEMORY_LIMIT],
            two_words: HashMap::new(),
        }
    }

    pub fn assemble(mut self, statements: Vec<Statement>) -> Result<[u8; ROM_MEMORY_LIMIT], Error> {
        for expression in statements {
            match expression {
                Statement::WordDefinitionStatement(label, value) =>
                    self.assemble_byte_definition(label, value),
                Statement::InstructionExprStmt(instruction) => {
                    self.add_instruction(instruction);
                    Ok(())
                },
                Statement::OrgStatement(TwoWordValue::Operand(TwoWordExpression::Literal(value))) => {
                    self.pc = value;
                    Ok(())
                },
                Statement::OrgStatement(TwoWordValue::Operand(TwoWordExpression::Label(label_value))) => {
                    if let Some(&value) = self.two_words.get(&label_value) {
                        self.pc = value;
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Statement::OrgStatement(
                    TwoWordValue::Sum(TwoWordExpression::Literal(op1), TwoWordExpression::Literal(op2))
                ) => {
                    self.pc = op1.wrapping_add(op2);
                    Ok(())
                },
                Statement::OrgStatement(
                    TwoWordValue::Sum(TwoWordExpression::Label(label), TwoWordExpression::Literal(op))
                ) |
                Statement::OrgStatement(
                    TwoWordValue::Sum(TwoWordExpression::Literal(op), TwoWordExpression::Label(label))
                ) => {
                    if let Some(&label_op) = self.labels.get(&label) {
                        self.pc = op.wrapping_add(label_op);
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Statement::OrgStatement(
                    TwoWordValue::Sum(TwoWordExpression::Label(op1), TwoWordExpression::Label(op2))
                ) => {
                    if let (Some(&op1), Some(&op2)) =
                        (self.labels.get(&op1), self.labels.get(&op2)) {
                        self.pc = op1.wrapping_add(op2);
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Statement::TwoWordDefinitionStatement(label, value) => self.assemble_word_definition(label, value),
                Statement::LabelDefinitionStatement(label) => {
                    self.labels.insert(label, self.pc);
                    Ok(())
                },
                _ => panic!("Not implemented yet!"),
            }?
        }
        Ok(self.rom)
    }

    #[inline]
    fn assemble_byte_definition(&mut self, label: LabelExpression, value: WordValue)
        -> Result<(), AssemblerError> {
        match value {
            WordValue::Operand(WordExpression::Literal(value)) => {
                self.words.insert(label, value);
                Ok(())
            },
            WordValue::Operand(WordExpression::Label(label_value)) => {
                if let Some(&value) = self.words.get(&label_value) {
                    self.words.insert(label, value);
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            WordValue::Rest(WordExpression::Literal(op1), WordExpression::Literal(op2)) => {
                self.words.insert(label, op1.wrapping_sub(op2));
                Ok(())
            },
            WordValue::Rest(WordExpression::Label(op_label), WordExpression::Literal(op)) |
            WordValue::Rest(WordExpression::Literal(op), WordExpression::Label(op_label)) => {
                if let Some(&op_label) = self.words.get(&op_label) {
                    self.words.insert(label, op.wrapping_sub(op_label));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            WordValue::Rest(WordExpression::Label(op1), WordExpression::Label(op2)) => {
                if let (Some(&op1), Some(&op2)) =
                (self.words.get(&op1), self.words.get(&op2)) {
                    self.words.insert(label, op1.wrapping_sub(op2));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            WordValue::Sum(WordExpression::Literal(op1), WordExpression::Literal(op2)) => {
                self.words.insert(label, op1.wrapping_add(op2));
                Ok(())
            },
            WordValue::Sum(WordExpression::Label(op_label), WordExpression::Literal(op)) |
            WordValue::Sum(WordExpression::Literal(op), WordExpression::Label(op_label)) => {
                if let Some(&op_label) = self.words.get(&op_label) {
                    self.words.insert(label, op.wrapping_add(op_label));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            WordValue::Sum(WordExpression::Label(op1), WordExpression::Label(op2)) => {
                if let (Some(&op1), Some(&op2)) =
                (self.words.get(&op1), self.words.get(&op2)) {
                    self.words.insert(label, op1.wrapping_add(op2));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
        }
    }

    #[inline]
    fn assemble_word_definition(&mut self, label: LabelExpression, value: TwoWordValue)
        -> Result<(), AssemblerError> {
        match value {
            TwoWordValue::Operand(TwoWordExpression::Literal(value)) => {
                self.two_words.insert(label, value);
                Ok(())
            },
            TwoWordValue::Operand(TwoWordExpression::Label(label_value)) => {
                if let Some(&value) = self.two_words.get(&label_value) {
                    self.two_words.insert(label, value);
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            TwoWordValue::Rest(TwoWordExpression::Literal(op1), TwoWordExpression::Literal(op2)) => {
                self.two_words.insert(label, op1.wrapping_sub(op2));
                Ok(())
            },
            TwoWordValue::Rest(TwoWordExpression::Label(op_label), TwoWordExpression::Literal(op)) |
            TwoWordValue::Rest(TwoWordExpression::Literal(op), TwoWordExpression::Label(op_label)) => {
                if let Some(&op_label) = self.two_words.get(&op_label) {
                    self.two_words.insert(label, op.wrapping_sub(op_label));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            TwoWordValue::Rest(TwoWordExpression::Label(op1), TwoWordExpression::Label(op2)) => {
                if let (Some(&op1), Some(&op2)) =
                    (self.two_words.get(&op1), self.two_words.get(&op2)) {
                    self.two_words.insert(label, op1.wrapping_sub(op2));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            TwoWordValue::Sum(TwoWordExpression::Literal(op1), TwoWordExpression::Literal(op2)) => {
                self.two_words.insert(label, op1.wrapping_add(op2));
                Ok(())
            },
            TwoWordValue::Sum(TwoWordExpression::Label(op_label), TwoWordExpression::Literal(op)) |
            TwoWordValue::Sum(TwoWordExpression::Literal(op), TwoWordExpression::Label(op_label)) => {
                if let Some(&op_label) = self.two_words.get(&op_label) {
                    self.two_words.insert(label, op.wrapping_add(op_label));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
            TwoWordValue::Sum(TwoWordExpression::Label(op1), TwoWordExpression::Label(op2)) => {
                if let (Some(&op1), Some(&op2)) =
                    (self.two_words.get(&op1), self.two_words.get(&op2)) {
                    self.two_words.insert(label, op1.wrapping_add(op2));
                    Ok(())
                } else {
                    Err(AssemblerError::LabelDoesntExist)
                }
            },
        }
    }

    fn word_value_to_u8(&self, value: WordValue) -> u8 {
        match value {
            WordValue::Operand(WordExpression::Label(l)) =>
                (*self.words.get(&l).unwrap()),
            WordValue::Operand(WordExpression::Literal(res)) => res,
            _ => panic!("Not implemented yet"),
        }
    }

    fn two_word_value_to_u16(&self, value: TwoWordValue) -> u16 {
        match value {
            TwoWordValue::Operand(TwoWordExpression::Label(l)) =>
                (*self.two_words.get(&l).unwrap()),
            TwoWordValue::Operand(TwoWordExpression::Literal(res)) => res,
            _ => panic!("Not implemented yet"),
        }
    }

    fn add_instruction(&mut self, instruction: Instruction) {
        for byte in self.bytes_for_instruction(instruction) {
            self.rom[self.pc as usize] = byte;
            self.pc += 1;
        }
    }

    fn add_lxi_instruction(&self, res: &mut Vec<u8>, register: RegisterType, two_words: TwoWordValue) {
        let opcode = match register {
            RegisterType::B => 0x01,
            RegisterType::D => 0x11,
            RegisterType::H => 0x21,
            RegisterType::Sp => 0x31,
            _ => panic!("Not implemented yet")
        };
        let byte = self.two_word_value_to_u16(two_words);
        res.push(opcode);
        res.push((byte & 0x0f) as u8);
        res.push(((byte & 0xf0) >> 8) as u8);
    }

    fn add_stax_instruction(&self, res: &mut Vec<u8>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x02,
            RegisterType::D => 0x12,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_inx_instruction(&self, res: &mut Vec<u8>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x03,
            RegisterType::D => 0x13,
            RegisterType::H => 0x23,
            RegisterType::Sp => 0x33,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_inr_instruction(&self, res: &mut Vec<u8>, location: Location) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x04,
            Location::Register { register: RegisterType::C } => 0x0c,
            Location::Register { register: RegisterType::D } => 0x14,
            Location::Register { register: RegisterType::E } => 0x1c,
            Location::Register { register: RegisterType::H } => 0x24,
            Location::Register { register: RegisterType::L } => 0x2c,
            Location::Memory => 0x34,
            Location::Register { register: RegisterType::A } => 0x3c,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_dcr_instruction(&self, res: &mut Vec<u8>, location: Location) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x05,
            Location::Register { register: RegisterType::C } => 0x0d,
            Location::Register { register: RegisterType::D } => 0x15,
            Location::Register { register: RegisterType::E } => 0x1d,
            Location::Register { register: RegisterType::H } => 0x25,
            Location::Register { register: RegisterType::L } => 0x2d,
            Location::Memory => 0x35,
            Location::Register { register: RegisterType::A } => 0x3d,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_mvi_instruction(&self, res: &mut Vec<u8>, location: Location, word: WordValue) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x06,
            Location::Register { register: RegisterType::C } => 0x0e,
            Location::Register { register: RegisterType::D } => 0x16,
            Location::Register { register: RegisterType::E } => 0x1e,
            Location::Register { register: RegisterType::H } => 0x26,
            Location::Register { register: RegisterType::L } => 0x2e,
            Location::Memory => 0x36,
            Location::Register { register: RegisterType::A } => 0x3e,
            _ => panic!("Not implemented yet")
        };
        let byte = self.word_value_to_u8(word);
        res.push(opcode);
        res.push(byte);
    }

    fn add_dad_instruction(&self, res: &mut Vec<u8>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x09,
            RegisterType::D => 0x19,
            RegisterType::H => 0x29,
            RegisterType::Sp => 0x39,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_ldax_instruction(&self, res: &mut Vec<u8>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x0a,
            RegisterType::D => 0x1a,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_dcx_instruction(&self, res: &mut Vec<u8>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x0b,
            RegisterType::D => 0x1b,
            RegisterType::H => 0x2b,
            RegisterType::Sp => 0x3b,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_simple_two_word_instruction(&self, opcode: u8, res: &mut Vec<u8>, value: TwoWordValue) {
        let two_word = self.two_word_value_to_u16(value);
        res.push(opcode);
        res.push((two_word & 0x0f) as u8);
        res.push(((two_word & 0xf0) >> 8) as u8);
    }

    fn add_mov_instruction(&self, res: &mut Vec<u8>, source: Location, destiny: Location) {
        match (destiny, source) {
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::B }) => res.push(0x40),
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::C }) => res.push(0x41),
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::D }) => res.push(0x42),
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::E }) => res.push(0x43),
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::H }) => res.push(0x44),
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::L }) => res.push(0x45),
            (Location::Register { register: RegisterType::B },
                Location::Memory) => res.push(0x46),
            (Location::Register { register: RegisterType::B },
                Location::Register { register: RegisterType::A }) => res.push(0x47),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::B }) => res.push(0x48),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::C }) => res.push(0x49),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::D }) => res.push(0x4a),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::E }) => res.push(0x4b),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::H }) => res.push(0x4c),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::L }) => res.push(0x4d),
            (Location::Register { register: RegisterType::C },
                Location::Memory) => res.push(0x4e),
            (Location::Register { register: RegisterType::C },
                Location::Register { register: RegisterType::A }) => res.push(0x4f),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::B }) => res.push(0x50),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::C }) => res.push(0x51),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::D }) => res.push(0x52),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::E }) => res.push(0x53),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::H }) => res.push(0x54),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::L }) => res.push(0x55),
            (Location::Register { register: RegisterType::D },
                Location::Memory) => res.push(0x56),
            (Location::Register { register: RegisterType::D },
                Location::Register { register: RegisterType::A }) => res.push(0x57),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::B }) => res.push(0x58),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::C }) => res.push(0x59),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::D }) => res.push(0x5a),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::E }) => res.push(0x5b),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::H }) => res.push(0x5c),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::L }) => res.push(0x5d),
            (Location::Register { register: RegisterType::E },
                Location::Memory) => res.push(0x5e),
            (Location::Register { register: RegisterType::E },
                Location::Register { register: RegisterType::A }) => res.push(0x5f),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::B }) => res.push(0x60),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::C }) => res.push(0x61),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::D }) => res.push(0x62),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::E }) => res.push(0x63),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::H }) => res.push(0x64),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::L }) => res.push(0x65),
            (Location::Register { register: RegisterType::H },
                Location::Memory) => res.push(0x66),
            (Location::Register { register: RegisterType::H },
                Location::Register { register: RegisterType::A }) => res.push(0x67),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::B }) => res.push(0x68),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::C }) => res.push(0x69),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::D }) => res.push(0x6a),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::E }) => res.push(0x6b),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::H }) => res.push(0x6c),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::L }) => res.push(0x6d),
            (Location::Register { register: RegisterType::L },
                Location::Memory) => res.push(0x6e),
            (Location::Register { register: RegisterType::L },
                Location::Register { register: RegisterType::A }) => res.push(0x6f),
            (Location::Memory,
                Location::Register { register: RegisterType::B }) => res.push(0x70),
            (Location::Memory,
                Location::Register { register: RegisterType::C }) => res.push(0x71),
            (Location::Memory,
                Location::Register { register: RegisterType::D }) => res.push(0x72),
            (Location::Memory,
                Location::Register { register: RegisterType::E }) => res.push(0x73),
            (Location::Memory,
                Location::Register { register: RegisterType::H }) => res.push(0x74),
            (Location::Memory,
                Location::Register { register: RegisterType::L }) => res.push(0x75),
            (Location::Memory,
                Location::Memory) => res.push(0x76),
            (Location::Memory,
                Location::Register { register: RegisterType::A }) => res.push(0x77),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::B }) => res.push(0x78),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::C }) => res.push(0x79),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::D }) => res.push(0x7a),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::E }) => res.push(0x7b),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::H }) => res.push(0x7c),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::L }) => res.push(0x7d),
            (Location::Register { register: RegisterType::A },
                Location::Memory) => res.push(0x7e),
            (Location::Register { register: RegisterType::A },
                Location::Register { register: RegisterType::A }) => res.push(0x7f),
            _ => panic!("Not implemented yet"),
        }
    }

    fn add_add_instruction(&self, res: &mut Vec<u8>, location: Location) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x80,
            Location::Register { register: RegisterType::C } => 0x81,
            Location::Register { register: RegisterType::D } => 0x82,
            Location::Register { register: RegisterType::E } => 0x83,
            Location::Register { register: RegisterType::H } => 0x84,
            Location::Register { register: RegisterType::L } => 0x85,
            Location::Memory => 0x86,
            Location::Register { register: RegisterType::A } => 0x87,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_adc_instruction(&self, res: &mut Vec<u8>, location: Location) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x88,
            Location::Register { register: RegisterType::C } => 0x89,
            Location::Register { register: RegisterType::D } => 0x8a,
            Location::Register { register: RegisterType::E } => 0x8b,
            Location::Register { register: RegisterType::H } => 0x8c,
            Location::Register { register: RegisterType::L } => 0x8d,
            Location::Memory => 0x8e,
            Location::Register { register: RegisterType::A } => 0x8f,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_sub_instruction(&self, res: &mut Vec<u8>, location: Location) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x90,
            Location::Register { register: RegisterType::C } => 0x91,
            Location::Register { register: RegisterType::D } => 0x92,
            Location::Register { register: RegisterType::E } => 0x93,
            Location::Register { register: RegisterType::H } => 0x94,
            Location::Register { register: RegisterType::L } => 0x95,
            Location::Memory => 0x96,
            Location::Register { register: RegisterType::A } => 0x97,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn add_sbb_instruction(&self, res: &mut Vec<u8>, location: Location) {
        let opcode = match location {
            Location::Register { register: RegisterType::B } => 0x98,
            Location::Register { register: RegisterType::C } => 0x99,
            Location::Register { register: RegisterType::D } => 0x9a,
            Location::Register { register: RegisterType::E } => 0x9b,
            Location::Register { register: RegisterType::H } => 0x9c,
            Location::Register { register: RegisterType::L } => 0x9d,
            Location::Memory => 0x9e,
            Location::Register { register: RegisterType::A } => 0x9f,
            _ => panic!("Not implemented yet")
        };
        res.push(opcode);
    }

    fn bytes_for_instruction(&self, instruction: Instruction) -> Vec<u8> {
        let mut res = Vec::with_capacity(3);
        match instruction {
            Instruction(InstructionCode::Noop, _, _) => res.push(0x00),
            Instruction(
                InstructionCode::Lxi,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                Some(InstructionArgument::TwoWord(v))
            ) => self.add_lxi_instruction(&mut res, register, v),
            Instruction(
                InstructionCode::Stax,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _
            ) => self.add_stax_instruction(&mut res, register),
            Instruction(
                InstructionCode::Inx,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _
            ) => self.add_inx_instruction(&mut res, register),
            Instruction(InstructionCode::Inr, Some(InstructionArgument::DataStore(location)), _) =>
                self.add_inr_instruction(&mut res, location),
            Instruction(InstructionCode::Dcr, Some(InstructionArgument::DataStore(location)), _) =>
                self.add_dcr_instruction(&mut res, location),
            Instruction(
                InstructionCode::Mvi,
                Some(InstructionArgument::DataStore(location)),
                Some(InstructionArgument::Word(v)),
            ) => self.add_mvi_instruction(&mut res, location, v),
            Instruction(InstructionCode::Rlc, _, _) => res.push(0x07),
            Instruction(
                InstructionCode::Dad,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _
            ) => self.add_dad_instruction(&mut res, register),
            Instruction(
                InstructionCode::Ldax,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _
            ) => self.add_ldax_instruction(&mut res, register),
            Instruction(
                InstructionCode::Dcx,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _
            ) => self.add_dcx_instruction(&mut res, register),
            Instruction(InstructionCode::Rrc, _, _) => res.push(0x0f),
            Instruction(InstructionCode::Ral, _, _) => res.push(0x17),
            Instruction(InstructionCode::Rar, _, _) => res.push(0x1f),
            Instruction(
                InstructionCode::Shld,
                Some(InstructionArgument::TwoWord(v)),
                _
            ) => self.add_simple_two_word_instruction(0x22, &mut res, v),
            Instruction(InstructionCode::Daa, _, _) => res.push(0x27),
            Instruction(
                InstructionCode::Lhld,
                Some(InstructionArgument::TwoWord(v)),
                _
            ) => self.add_simple_two_word_instruction(0x2a, &mut res, v),
            Instruction(InstructionCode::Cma, _, _) => res.push(0x2f),
            Instruction(
                InstructionCode::Sta,
                Some(InstructionArgument::TwoWord(v)),
                _
            ) => self.add_simple_two_word_instruction(0x32, &mut res, v),
            Instruction(InstructionCode::Stc, _, _) => res.push(0x37),
            Instruction(
                InstructionCode::Lda,
                Some(InstructionArgument::TwoWord(v)),
                _
            ) => self.add_simple_two_word_instruction(0x3a, &mut res, v),
            Instruction(InstructionCode::Cmc, _, _) => res.push(0x3f),
            Instruction(
                InstructionCode::Mov,
                Some(InstructionArgument::DataStore(d)),
                Some(InstructionArgument::DataStore(s))
            ) => self.add_mov_instruction(&mut res, s, d),
            Instruction(InstructionCode::Add, Some(InstructionArgument::DataStore(location)), _) =>
                self.add_add_instruction(&mut res, location),
            Instruction(InstructionCode::Adc, Some(InstructionArgument::DataStore(location)), _) =>
                self.add_adc_instruction(&mut res, location),
            Instruction(InstructionCode::Sub, Some(InstructionArgument::DataStore(location)), _) =>
                self.add_sub_instruction(&mut res, location),
            Instruction(InstructionCode::Sbb, Some(InstructionArgument::DataStore(location)), _) =>
                self.add_sbb_instruction(&mut res, location),
            /*
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::B } } =>
                        res.push(0xa0),
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::C } } =>
                        res.push(0xa1),
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::D } } =>
                        res.push(0xa2),
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::E } } =>
                        res.push(0xa3),
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::H } } =>
                        res.push(0xa4),
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::L } } =>
                        res.push(0xa5),
                    Intel8080Instruction::Ana { source: Location::Memory } =>
                        res.push(0xa6),
                    Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::A } } =>
                        res.push(0xa7),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::B } } =>
                        res.push(0xa8),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::C } } =>
                        res.push(0xa9),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::D } } =>
                        res.push(0xaa),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::E } } =>
                        res.push(0xab),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::H } } =>
                        res.push(0xac),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::L } } =>
                        res.push(0xad),
                    Intel8080Instruction::Xra { source: Location::Memory } =>
                        res.push(0xae),
                    Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::A } } =>
                        res.push(0xaf),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::B } } =>
                        res.push(0xb0),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::C } } =>
                        res.push(0xb1),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::D } } =>
                        res.push(0xb2),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::E } } =>
                        res.push(0xb3),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::H } } =>
                        res.push(0xb4),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::L } } =>
                        res.push(0xb5),
                    Intel8080Instruction::Ora { source: Location::Memory } =>
                        res.push(0xb6),
                    Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::A } } =>
                        res.push(0xb7),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::B } } =>
                        res.push(0xb8),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::C } } =>
                        res.push(0xb9),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::D } } =>
                        res.push(0xba),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::E } } =>
                        res.push(0xbb),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::H } } =>
                        res.push(0xbc),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::L } } =>
                        res.push(0xbd),
                    Intel8080Instruction::Cmp { source: Location::Memory } =>
                        res.push(0xbe),
                    Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::A } } =>
                        res.push(0xbf),
                    Intel8080Instruction::Rnz => res.push(0xc0),
                    Intel8080Instruction::Pop { register: RegisterType::B } => res.push(0xc1),
                    Intel8080Instruction::Jnz { address: [low_byte, high_byte] } => {
                        res.push(0xc2);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Jmp { address: [low_byte, high_byte] } => {
                        res.push(0xc3);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Cnz { address: [low_byte, high_byte] } => {
                        res.push(0xc4);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Push { register: RegisterType::B } => res.push(0xc5),
                    Intel8080Instruction::Adi { byte } => {
                        res.push(0xc6);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 0 } => res.push(0xc7),
                    Intel8080Instruction::Rz => res.push(0xc8),
                    Intel8080Instruction::Ret => res.push(0xc9),
                    Intel8080Instruction::Jz { address: [low_byte, high_byte] } => {
                        res.push(0xca);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Cz { address: [low_byte, high_byte] } => {
                        res.push(0xcc);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Call { address: [low_byte, high_byte] } => {
                        res.push(0xcd);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Aci { byte } => {
                        res.push(0xce);
                        res.push(byte)
                    },
                    Intel8080Instruction::Rst { byte: 1 } => res.push(0xcf),
                    Intel8080Instruction::Rnc => res.push(0xd0),
                    Intel8080Instruction::Pop { register: RegisterType::D } => res.push(0xd1),
                    Intel8080Instruction::Jnc { address: [low_byte, high_byte] } => {
                        res.push(0xd2);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Out { byte } => {
                        res.push(0xd3);
                        res.push(byte);
                    },
                    Intel8080Instruction::Cnc { address: [high_byte, low_byte] } => {
                        res.push(0xd4);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Push { register: RegisterType::D } => res.push(0xd5),
                    Intel8080Instruction::Sui { byte } => {
                        res.push(0xd6);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 2 } => res.push(0xd7),
                    Intel8080Instruction::Rc => res.push(0xd8),
                    Intel8080Instruction::Jc { address: [low_byte, high_byte] } => {
                        res.push(0xda);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::In { byte } => {
                        res.push(0xdb);
                        res.push(byte);
                    },
                    Intel8080Instruction::Cc { address: [low_byte, high_byte] } => {
                        res.push(0xdc);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Sbi { byte } => {
                        res.push(0xde);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 3 } => res.push(0xdf),
                    Intel8080Instruction::Rpo => res.push(0xe0),
                    Intel8080Instruction::Pop { register: RegisterType::H } => res.push(0xe1),
                    Intel8080Instruction::Jpo { address: [low_byte, high_byte] } => {
                        res.push(0xe2);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Xthl => res.push(0xe3),
                    Intel8080Instruction::Cpo { address: [low_byte, high_byte] } => {
                        res.push(0xe4);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Push { register: RegisterType::H } => res.push(0xe5),
                    Intel8080Instruction::Ani { byte } => {
                        res.push(0xe6);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 4 } => res.push(0xe7),
                    Intel8080Instruction::Rpe => res.push(0xe8),
                    Intel8080Instruction::Pchl => res.push(0xe9),
                    Intel8080Instruction::Jpe { address: [low_byte, high_byte] } => {
                        res.push(0xea);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Xchg => res.push(0xeb),
                    Intel8080Instruction::Cpe { address: [low_byte, high_byte] } => {
                        res.push(0xec);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Xri { byte } => {
                        res.push(0xee);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 5 } => res.push(0xef),
                    Intel8080Instruction::Rp => res.push(0xf0),
                    Intel8080Instruction::Pop { register: RegisterType::Psw } => res.push(0xf1),
                    Intel8080Instruction::Jp { address: [low_byte, high_byte] } => {
                        res.push(0xf2);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Di => res.push(0xf3),
                    Intel8080Instruction::Cp { address: [low_byte, high_byte] } => {
                        res.push(0xf4);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Push { register: RegisterType::Psw } => res.push(0xf5),
                    Intel8080Instruction::Ori { byte } => {
                        res.push(0xf6);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 6 } => res.push(0xf7),
                    Intel8080Instruction::Rm => res.push(0xf8),
                    Intel8080Instruction::Sphl => res.push(0xf9),
                    Intel8080Instruction::Jm { address: [low_byte, high_byte] } => {
                        res.push(0xfa);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Ei => res.push(0xfb),
                    Intel8080Instruction::Cm { address: [low_byte, high_byte] } => {
                        res.push(0xfc);
                        res.push(low_byte);
                        res.push(high_byte);
                    },
                    Intel8080Instruction::Cpi { byte } => {
                        res.push(0xfe);
                        res.push(byte);
                    },
                    Intel8080Instruction::Rst { byte: 7 } => res.push(0xff),
                    */
                    _ => panic!("unfined method"),
                }
                res
            }
}
