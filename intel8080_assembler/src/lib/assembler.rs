extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Instruction, Intel8080Instruction, Location, RegisterType, ROM_MEMORY_LIMIT};
use std::collections::HashMap;
use super::{AssemblerError, ByteOperand, ByteValue, Expression, Label, WordOperand, WordValue};

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
                Expression::ByteDefinition {
                    label,
                    value: ByteValue::Operand(ByteOperand::Literal(value)),
                } => {
                    self.bytes.insert(label, value);
                    Ok(())
                },
                Expression::ByteDefinition {
                    label,
                    value: ByteValue::Operand(ByteOperand::Label(label_value)),
                } => {
                    if let Some(&value) = self.bytes.get(&label_value) {
                        self.bytes.insert(label, value);
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Expression::Instruction(instruction) => {
                    self.pc += instruction.size()? as u16;
                    self.add_instruction(instruction);
                    Ok(())
                },
                Expression::OrgStatement(WordValue::Operand(WordOperand::Literal(value))) => {
                    self.pc = value;
                    Ok(())
                },
                Expression::OrgStatement(WordValue::Operand(WordOperand::Label(label_value))) => {
                    if let Some(&value) = self.words.get(&label_value) {
                        self.pc = value;
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Expression::OrgStatement(
                    WordValue::Sum(WordOperand::Literal(op1), WordOperand::Literal(op2))
                ) => {
                    self.pc = op1.wrapping_add(op2);
                    Ok(())
                },
                Expression::OrgStatement(
                    WordValue::Sum(WordOperand::Label(label), WordOperand::Literal(op))
                ) |
                Expression::OrgStatement(
                    WordValue::Sum(WordOperand::Literal(op), WordOperand::Label(label))
                ) => {
                    if let Some(&label_op) = self.labels.get(&label) {
                        self.pc = op.wrapping_add(label_op);
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Expression::OrgStatement(
                    WordValue::Sum(WordOperand::Label(op1), WordOperand::Label(op2))
                ) => {
                    if let (Some(&op1), Some(&op2)) =
                        (self.labels.get(&op1), self.labels.get(&op2)) {
                        self.pc = op1.wrapping_add(op2);
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Expression::LabelDefinition(label) => {
                    self.labels.insert(label, self.pc);
                    Ok(())
                },
                Expression::WordDefinition {
                    label,
                    value: WordValue::Operand(WordOperand::Literal(value)),
                } => {
                    self.words.insert(label, value);
                    Ok(())
                },
                Expression::WordDefinition {
                    label,
                    value: WordValue::Operand(WordOperand::Label(label_value)),
                } => {
                    if let Some(&value) = self.words.get(&label_value) {
                        self.words.insert(label, value);
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Expression::WordDefinition {
                    label,
                    value: WordValue::Sum(WordOperand::Literal(op1), WordOperand::Literal(op2)),
                } => {
                    self.words.insert(label, op1.wrapping_add(op2));
                    Ok(())
                },
                Expression::WordDefinition {
                    label,
                    value: WordValue::Sum(WordOperand::Label(op_label), WordOperand::Literal(op)),
                } |
                Expression::WordDefinition {
                    label,
                    value: WordValue::Sum(WordOperand::Literal(op), WordOperand::Label(op_label)),
                } => {
                    if let Some(&op_label) = self.words.get(&op_label) {
                        self.words.insert(label, op.wrapping_add(op_label));
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
                Expression::WordDefinition {
                    label,
                    value: WordValue::Sum(WordOperand::Label(op1), WordOperand::Label(op2)),
                } => {
                    if let (Some(&op1), Some(&op2)) =
                        (self.labels.get(&op1), self.labels.get(&op2)) {
                        self.words.insert(label, op1.wrapping_add(op2));
                        Ok(())
                    } else {
                        Err(AssemblerError::LabelDoesntExist)
                    }
                },
            }?
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
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::B } } =>
                res.push(0x80),
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::C } } =>
                res.push(0x81),
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::D } } =>
                res.push(0x82),
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::E } } =>
                res.push(0x83),
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::H } } =>
                res.push(0x84),
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::L } } =>
                res.push(0x85),
            Intel8080Instruction::Add { source: Location::Memory } => res.push(0x86),
            Intel8080Instruction::Add { source: Location::Register { register: RegisterType::A } } =>
                res.push(0x87),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::B } } =>
                res.push(0x88),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::C } } =>
                res.push(0x89),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::D } } =>
                res.push(0x8a),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::E } } =>
                res.push(0x8b),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::H } } =>
                res.push(0x8c),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::L } } =>
                res.push(0x8d),
            Intel8080Instruction::Adc { source: Location::Memory } => res.push(0x8e),
            Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::A } } =>
                res.push(0x8f),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::B } } =>
                res.push(0x90),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::C } } =>
                res.push(0x91),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::D } } =>
                res.push(0x92),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::E } } =>
                res.push(0x93),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::H } } =>
                res.push(0x94),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::L } } =>
                res.push(0x95),
            Intel8080Instruction::Sub { source: Location::Memory } =>
                res.push(0x96),
            Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::A } } =>
                res.push(0x97),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::B } } =>
                res.push(0x98),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::C } } =>
                res.push(0x99),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::D } } =>
                res.push(0x9a),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::E } } =>
                res.push(0x9b),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::H } } =>
                res.push(0x9c),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::L } } =>
                res.push(0x9d),
            Intel8080Instruction::Sbb { source: Location::Memory } =>
                res.push(0x9e),
            Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::A } } =>
                res.push(0x9f),
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
            _ => panic!("unfined method"),
        }
        res
    }
}