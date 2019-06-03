extern crate failure;
extern crate intel8080cpu;

use super::*;
use failure::Error;
use intel8080cpu::{Location, RegisterType};
use std::collections::HashMap;

const ROM_MEMORY_LIMIT: usize = 65536;

#[derive(Clone, Debug, PartialEq)]
enum StageOneValue {
    ByteOperation(OperationExpression),
    OrgStatement(u16),
    TwoByteOperation(OperationExpression),
    Word(u8),
}

pub struct Assembler {
    pc: u16,
    stage_one_room: Vec<StageOneValue>,
    room: [u8; ROM_MEMORY_LIMIT],
    two_words: HashMap<LabelExpression, u16>,
}

impl Default for Assembler {
    fn default() -> Assembler {
        Assembler {
            pc: 0,
            room: [0; ROM_MEMORY_LIMIT],
            stage_one_room: Vec::with_capacity(ROM_MEMORY_LIMIT),
            two_words: HashMap::new(),
        }
    }
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler::default()
    }

    pub fn assemble(mut self, statements: Vec<Statement>) -> Result<[u8; ROM_MEMORY_LIMIT], Error> {
        self.stage_one(statements)?;
        self.stage_two()?;
        Ok(self.room)
    }

    fn stage_one(&mut self, statements: Vec<Statement>) -> Result<(), Error> {
        for expression in statements {
            match expression {
                Statement::InstructionExprStmt(instruction) => {
                    self.add_instruction(instruction)?;
                }
                Statement::LabelDefinitionStatement(label) => {
                    self.two_words.insert(label, self.pc);
                }
                Statement::OrgStatement(tw) => {
                    self.pc = tw;
                    self.stage_one_room.push(StageOneValue::OrgStatement(tw));
                }
                Statement::TwoWordDefinitionStatement(label, value) => {
                    let value = self.operation_to_u16(value)?;
                    self.two_words.insert(label, value);
                }
                Statement::WordDefinitionStatement(label, value) => {
                    let value = u16::from(self.operation_to_u8(value)?);
                    self.two_words.insert(label, value);
                }
            };
        }
        Ok(())
    }

    fn stage_two(&mut self) -> Result<(), Error> {
        let iter = self.stage_one_room.iter();
        self.pc = 0;
        for v in iter {
            match v {
                StageOneValue::ByteOperation(op) => {
                    self.room[self.pc as usize] = self.operation_to_u8(op.clone())?;
                    self.pc = self.pc.wrapping_add(1);
                }
                StageOneValue::OrgStatement(address) => self.pc = *address,
                StageOneValue::TwoByteOperation(op) => {
                    let tw = self.operation_to_u16(op.clone())?;
                    self.room[self.pc as usize] = (tw & 0x00ff) as u8;
                    self.pc = self.pc.wrapping_add(1);
                    self.room[self.pc as usize] = ((tw & 0xff00) >> 8) as u8;
                    self.pc = self.pc.wrapping_add(1);
                }
                StageOneValue::Word(b) => {
                    self.room[self.pc as usize] = *b;
                    self.pc = self.pc.wrapping_add(1);
                }
            }
        }
        Ok(())
    }

    fn operation_to_u8(&self, operation: OperationExpression) -> Result<u8, Error> {
        Ok(self.operation_to_u16(operation)? as u8)
    }

    fn operation_to_u16(&self, operation: OperationExpression) -> Result<u16, Error> {
        match operation {
            OperationExpression::And(left, right) => {
                Ok(self.operation_to_u16(*left)? & self.operation_to_u16(*right)?)
            }
            OperationExpression::Div(left, right) => Ok(self
                .operation_to_u16(*left)?
                .wrapping_div(self.operation_to_u16(*right)?)),
            OperationExpression::Group(op) => self.operation_to_u16(*op),
            OperationExpression::Not(op) => Ok(!self.operation_to_u16(*op)?),
            OperationExpression::Mod(left, right) => {
                Ok(self.operation_to_u16(*left)? % self.operation_to_u16(*right)?)
            }
            OperationExpression::Mult(left, right) => Ok(self
                .operation_to_u16(*left)?
                .wrapping_mul(self.operation_to_u16(*right)?)),
            OperationExpression::Operand(op) => self.operand_to_u16(op),
            OperationExpression::Or(left, right) => {
                Ok(self.operation_to_u16(*left)? | self.operation_to_u16(*right)?)
            }
            OperationExpression::Sub(left, right) => Ok(self
                .operation_to_u16(*left)?
                .wrapping_sub(self.operation_to_u16(*right)?)),
            OperationExpression::Shl(left, right) => Ok(self
                .operation_to_u16(*left)?
                .wrapping_shl(u32::from(self.operation_to_u16(*right)?))),
            OperationExpression::Shr(left, right) => Ok(self
                .operation_to_u16(*left)?
                .wrapping_shr(u32::from(self.operation_to_u16(*right)?))),
            OperationExpression::Sum(left, right) => Ok(self
                .operation_to_u16(*left)?
                .wrapping_add(self.operation_to_u16(*right)?)),
            OperationExpression::Xor(left, right) => {
                Ok(self.operation_to_u16(*left)? ^ self.operation_to_u16(*right)?)
            }
        }
    }

    fn operand_to_u16(&self, operand: TwoWordExpression) -> Result<u16, Error> {
        match operand {
            TwoWordExpression::Char(char_value) => Ok(char_value as u16),
            TwoWordExpression::Dollar => Ok(self.pc - 1),
            TwoWordExpression::Label(l) => self
                .two_words
                .get(&l)
                .copied()
                .ok_or_else(|| Error::from(AssemblerError::LabelNotFound { label: l })),
            TwoWordExpression::Literal(v) => Ok(v),
        }
    }

    fn add_instruction(&mut self, instruction: Instruction) -> Result<(), Error> {
        for v in self.bytes_for_instruction(instruction)? {
            let steps = match v {
                StageOneValue::OrgStatement(_) => 0,
                StageOneValue::ByteOperation(_) | StageOneValue::Word(_) => 1,
                _ => 2,
            };
            self.stage_one_room.push(v);
            self.pc = self.pc.wrapping_add(steps);
        }
        Ok(())
    }

    fn add_lxi_instruction(
        &self,
        res: &mut Vec<StageOneValue>,
        register: RegisterType,
        op: OperationExpression,
    ) -> Result<(), Error> {
        let opcode = match register {
            RegisterType::B => 0x01,
            RegisterType::D => 0x11,
            RegisterType::H => 0x21,
            RegisterType::Sp => 0x31,
            _ => panic!("Not implemented yet"),
        };
        self.add_simple_two_word_instruction(opcode, res, op)?;
        Ok(())
    }

    fn add_stax_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x02,
            RegisterType::D => 0x12,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_inx_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x03,
            RegisterType::D => 0x13,
            RegisterType::H => 0x23,
            RegisterType::Sp => 0x33,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_inr_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x04,
            Location::Register {
                register: RegisterType::C,
            } => 0x0c,
            Location::Register {
                register: RegisterType::D,
            } => 0x14,
            Location::Register {
                register: RegisterType::E,
            } => 0x1c,
            Location::Register {
                register: RegisterType::H,
            } => 0x24,
            Location::Register {
                register: RegisterType::L,
            } => 0x2c,
            Location::Memory => 0x34,
            Location::Register {
                register: RegisterType::A,
            } => 0x3c,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_dcr_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x05,
            Location::Register {
                register: RegisterType::C,
            } => 0x0d,
            Location::Register {
                register: RegisterType::D,
            } => 0x15,
            Location::Register {
                register: RegisterType::E,
            } => 0x1d,
            Location::Register {
                register: RegisterType::H,
            } => 0x25,
            Location::Register {
                register: RegisterType::L,
            } => 0x2d,
            Location::Memory => 0x35,
            Location::Register {
                register: RegisterType::A,
            } => 0x3d,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_mvi_instruction(
        &self,
        res: &mut Vec<StageOneValue>,
        location: Location,
        op: OperationExpression,
    ) -> Result<(), Error> {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x06,
            Location::Register {
                register: RegisterType::C,
            } => 0x0e,
            Location::Register {
                register: RegisterType::D,
            } => 0x16,
            Location::Register {
                register: RegisterType::E,
            } => 0x1e,
            Location::Register {
                register: RegisterType::H,
            } => 0x26,
            Location::Register {
                register: RegisterType::L,
            } => 0x2e,
            Location::Memory => 0x36,
            Location::Register {
                register: RegisterType::A,
            } => 0x3e,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
        res.push(StageOneValue::ByteOperation(op));
        Ok(())
    }

    fn add_dad_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x09,
            RegisterType::D => 0x19,
            RegisterType::H => 0x29,
            RegisterType::Sp => 0x39,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_ldax_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x0a,
            RegisterType::D => 0x1a,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_dcx_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0x0b,
            RegisterType::D => 0x1b,
            RegisterType::H => 0x2b,
            RegisterType::Sp => 0x3b,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_simple_two_word_instruction(
        &self,
        opcode: u8,
        res: &mut Vec<StageOneValue>,
        op: OperationExpression,
    ) -> Result<(), Error> {
        res.push(StageOneValue::Word(opcode));
        res.push(StageOneValue::TwoByteOperation(op));
        Ok(())
    }

    fn add_simple_word_instruction(
        &self,
        opcode: u8,
        res: &mut Vec<StageOneValue>,
        op: OperationExpression,
    ) -> Result<(), Error> {
        res.push(StageOneValue::Word(opcode));
        res.push(StageOneValue::ByteOperation(op));
        Ok(())
    }

    fn add_mov_instruction(
        &self,
        res: &mut Vec<StageOneValue>,
        source: Location,
        destiny: Location,
    ) {
        match (destiny, source) {
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x40)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x41)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x42)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x43)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x44)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x45)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x46)),
            (
                Location::Register {
                    register: RegisterType::B,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x47)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x48)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x49)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x4a)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x4b)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x4c)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x4d)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x4e)),
            (
                Location::Register {
                    register: RegisterType::C,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x4f)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x50)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x51)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x52)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x53)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x54)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x55)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x56)),
            (
                Location::Register {
                    register: RegisterType::D,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x57)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x58)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x59)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x5a)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x5b)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x5c)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x5d)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x5e)),
            (
                Location::Register {
                    register: RegisterType::E,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x5f)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x60)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x61)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x62)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x63)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x64)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x65)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x66)),
            (
                Location::Register {
                    register: RegisterType::H,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x67)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x68)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x69)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x6a)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x6b)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x6c)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x6d)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x6e)),
            (
                Location::Register {
                    register: RegisterType::L,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x6f)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x70)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x71)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x72)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x73)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x74)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x75)),
            (Location::Memory, Location::Memory) => res.push(StageOneValue::Word(0x76)),
            (
                Location::Memory,
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x77)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::B,
                },
            ) => res.push(StageOneValue::Word(0x78)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::C,
                },
            ) => res.push(StageOneValue::Word(0x79)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::D,
                },
            ) => res.push(StageOneValue::Word(0x7a)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::E,
                },
            ) => res.push(StageOneValue::Word(0x7b)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::H,
                },
            ) => res.push(StageOneValue::Word(0x7c)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::L,
                },
            ) => res.push(StageOneValue::Word(0x7d)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Memory,
            ) => res.push(StageOneValue::Word(0x7e)),
            (
                Location::Register {
                    register: RegisterType::A,
                },
                Location::Register {
                    register: RegisterType::A,
                },
            ) => res.push(StageOneValue::Word(0x7f)),
            _ => panic!("Not implemented yet"),
        }
    }

    fn add_add_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x80,
            Location::Register {
                register: RegisterType::C,
            } => 0x81,
            Location::Register {
                register: RegisterType::D,
            } => 0x82,
            Location::Register {
                register: RegisterType::E,
            } => 0x83,
            Location::Register {
                register: RegisterType::H,
            } => 0x84,
            Location::Register {
                register: RegisterType::L,
            } => 0x85,
            Location::Memory => 0x86,
            Location::Register {
                register: RegisterType::A,
            } => 0x87,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_adc_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x88,
            Location::Register {
                register: RegisterType::C,
            } => 0x89,
            Location::Register {
                register: RegisterType::D,
            } => 0x8a,
            Location::Register {
                register: RegisterType::E,
            } => 0x8b,
            Location::Register {
                register: RegisterType::H,
            } => 0x8c,
            Location::Register {
                register: RegisterType::L,
            } => 0x8d,
            Location::Memory => 0x8e,
            Location::Register {
                register: RegisterType::A,
            } => 0x8f,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_sub_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x90,
            Location::Register {
                register: RegisterType::C,
            } => 0x91,
            Location::Register {
                register: RegisterType::D,
            } => 0x92,
            Location::Register {
                register: RegisterType::E,
            } => 0x93,
            Location::Register {
                register: RegisterType::H,
            } => 0x94,
            Location::Register {
                register: RegisterType::L,
            } => 0x95,
            Location::Memory => 0x96,
            Location::Register {
                register: RegisterType::A,
            } => 0x97,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_sbb_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0x98,
            Location::Register {
                register: RegisterType::C,
            } => 0x99,
            Location::Register {
                register: RegisterType::D,
            } => 0x9a,
            Location::Register {
                register: RegisterType::E,
            } => 0x9b,
            Location::Register {
                register: RegisterType::H,
            } => 0x9c,
            Location::Register {
                register: RegisterType::L,
            } => 0x9d,
            Location::Memory => 0x9e,
            Location::Register {
                register: RegisterType::A,
            } => 0x9f,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_ana_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0xa0,
            Location::Register {
                register: RegisterType::C,
            } => 0xa1,
            Location::Register {
                register: RegisterType::D,
            } => 0xa2,
            Location::Register {
                register: RegisterType::E,
            } => 0xa3,
            Location::Register {
                register: RegisterType::H,
            } => 0xa4,
            Location::Register {
                register: RegisterType::L,
            } => 0xa5,
            Location::Memory => 0xa6,
            Location::Register {
                register: RegisterType::A,
            } => 0xa7,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_xra_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0xa8,
            Location::Register {
                register: RegisterType::C,
            } => 0xa9,
            Location::Register {
                register: RegisterType::D,
            } => 0xaa,
            Location::Register {
                register: RegisterType::E,
            } => 0xab,
            Location::Register {
                register: RegisterType::H,
            } => 0xac,
            Location::Register {
                register: RegisterType::L,
            } => 0xad,
            Location::Memory => 0xae,
            Location::Register {
                register: RegisterType::A,
            } => 0xaf,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_ora_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0xb0,
            Location::Register {
                register: RegisterType::C,
            } => 0xb1,
            Location::Register {
                register: RegisterType::D,
            } => 0xb2,
            Location::Register {
                register: RegisterType::E,
            } => 0xb3,
            Location::Register {
                register: RegisterType::H,
            } => 0xb4,
            Location::Register {
                register: RegisterType::L,
            } => 0xb5,
            Location::Memory => 0xb6,
            Location::Register {
                register: RegisterType::A,
            } => 0xb7,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_cmp_instruction(&self, res: &mut Vec<StageOneValue>, location: Location) {
        let opcode = match location {
            Location::Register {
                register: RegisterType::B,
            } => 0xb8,
            Location::Register {
                register: RegisterType::C,
            } => 0xb9,
            Location::Register {
                register: RegisterType::D,
            } => 0xba,
            Location::Register {
                register: RegisterType::E,
            } => 0xbb,
            Location::Register {
                register: RegisterType::H,
            } => 0xbc,
            Location::Register {
                register: RegisterType::L,
            } => 0xbd,
            Location::Memory => 0xbe,
            Location::Register {
                register: RegisterType::A,
            } => 0xbf,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_pop_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0xc1,
            RegisterType::D => 0xd1,
            RegisterType::H => 0xe1,
            RegisterType::Psw => 0xf1,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_push_instruction(&self, res: &mut Vec<StageOneValue>, register: RegisterType) {
        let opcode = match register {
            RegisterType::B => 0xc5,
            RegisterType::D => 0xd5,
            RegisterType::H => 0xe5,
            RegisterType::Psw => 0xf5,
            _ => panic!("Not implemented yet"),
        };
        res.push(StageOneValue::Word(opcode));
    }

    fn add_rst_instruction(
        &self,
        res: &mut Vec<StageOneValue>,
        op: OperationExpression,
    ) -> Result<(), Error> {
        match self.operation_to_u8(op)? {
            0 => res.push(StageOneValue::Word(0xc7)),
            1 => res.push(StageOneValue::Word(0xcf)),
            2 => res.push(StageOneValue::Word(0xd7)),
            3 => res.push(StageOneValue::Word(0xdf)),
            4 => res.push(StageOneValue::Word(0xe7)),
            5 => res.push(StageOneValue::Word(0xef)),
            6 => res.push(StageOneValue::Word(0xf7)),
            7 => res.push(StageOneValue::Word(0xff)),
            _ => panic!("Not implemented yet"),
        };
        Ok(())
    }

    fn bytes_for_instruction(&self, instruction: Instruction) -> Result<Vec<StageOneValue>, Error> {
        let mut res = Vec::with_capacity(3);
        match instruction {
            Instruction(InstructionCode::Noop, _, _) => res.push(StageOneValue::Word(0x00)),
            Instruction(
                InstructionCode::Lxi,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                Some(InstructionArgument::TwoWord(v)),
            ) => self.add_lxi_instruction(&mut res, register, v)?,
            Instruction(
                InstructionCode::Stax,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_stax_instruction(&mut res, register),
            Instruction(
                InstructionCode::Inx,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_inx_instruction(&mut res, register),
            Instruction(
                InstructionCode::Inr,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_inr_instruction(&mut res, location),
            Instruction(
                InstructionCode::Dcr,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_dcr_instruction(&mut res, location),
            Instruction(
                InstructionCode::Mvi,
                Some(InstructionArgument::DataStore(location)),
                Some(InstructionArgument::TwoWord(v)),
            ) => self.add_mvi_instruction(&mut res, location, v)?,
            Instruction(InstructionCode::Rlc, _, _) => res.push(StageOneValue::Word(0x07)),
            Instruction(
                InstructionCode::Dad,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_dad_instruction(&mut res, register),
            Instruction(
                InstructionCode::Ldax,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_ldax_instruction(&mut res, register),
            Instruction(
                InstructionCode::Dcx,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_dcx_instruction(&mut res, register),
            Instruction(InstructionCode::Rrc, _, _) => res.push(StageOneValue::Word(0x0f)),
            Instruction(InstructionCode::Ral, _, _) => res.push(StageOneValue::Word(0x17)),
            Instruction(InstructionCode::Rar, _, _) => res.push(StageOneValue::Word(0x1f)),
            Instruction(InstructionCode::Shld, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0x22, &mut res, v)?
            }
            Instruction(InstructionCode::Daa, _, _) => res.push(StageOneValue::Word(0x27)),
            Instruction(InstructionCode::Lhld, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0x2a, &mut res, v)?
            }
            Instruction(InstructionCode::Cma, _, _) => res.push(StageOneValue::Word(0x2f)),
            Instruction(InstructionCode::Sta, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0x32, &mut res, v)?
            }
            Instruction(InstructionCode::Stc, _, _) => res.push(StageOneValue::Word(0x37)),
            Instruction(InstructionCode::Lda, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0x3a, &mut res, v)?
            }
            Instruction(InstructionCode::Cmc, _, _) => res.push(StageOneValue::Word(0x3f)),
            Instruction(
                InstructionCode::Mov,
                Some(InstructionArgument::DataStore(d)),
                Some(InstructionArgument::DataStore(s)),
            ) => self.add_mov_instruction(&mut res, s, d),
            Instruction(
                InstructionCode::Add,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_add_instruction(&mut res, location),
            Instruction(
                InstructionCode::Adc,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_adc_instruction(&mut res, location),
            Instruction(
                InstructionCode::Sub,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_sub_instruction(&mut res, location),
            Instruction(
                InstructionCode::Sbb,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_sbb_instruction(&mut res, location),
            Instruction(
                InstructionCode::Ana,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_ana_instruction(&mut res, location),
            Instruction(
                InstructionCode::Xra,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_xra_instruction(&mut res, location),
            Instruction(
                InstructionCode::Ora,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_ora_instruction(&mut res, location),
            Instruction(
                InstructionCode::Cmp,
                Some(InstructionArgument::DataStore(location)),
                _,
            ) => self.add_cmp_instruction(&mut res, location),
            Instruction(InstructionCode::Rnz, _, _) => res.push(StageOneValue::Word(0xc0)),
            Instruction(
                InstructionCode::Pop,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_pop_instruction(&mut res, register),
            Instruction(InstructionCode::Jnz, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xc2, &mut res, v)?
            }
            Instruction(InstructionCode::Jmp, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xc3, &mut res, v)?
            }
            Instruction(InstructionCode::Cnz, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xc4, &mut res, v)?
            }
            Instruction(
                InstructionCode::Push,
                Some(InstructionArgument::DataStore(Location::Register { register })),
                _,
            ) => self.add_push_instruction(&mut res, register),
            Instruction(InstructionCode::Adi, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xc6, &mut res, v)?
            }
            Instruction(InstructionCode::Rst, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_rst_instruction(&mut res, v)?
            }
            Instruction(InstructionCode::Rz, _, _) => res.push(StageOneValue::Word(0xc8)),
            Instruction(InstructionCode::Ret, _, _) => res.push(StageOneValue::Word(0xc9)),
            Instruction(InstructionCode::Jz, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xca, &mut res, v)?
            }
            Instruction(InstructionCode::Cz, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xcc, &mut res, v)?
            }
            Instruction(InstructionCode::Call, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xcd, &mut res, v)?
            }
            Instruction(InstructionCode::Aci, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xce, &mut res, v)?
            }
            Instruction(InstructionCode::Rnc, _, _) => res.push(StageOneValue::Word(0xd0)),
            Instruction(InstructionCode::Jnc, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xd2, &mut res, v)?
            }
            Instruction(InstructionCode::Out, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xd3, &mut res, v)?
            }
            Instruction(InstructionCode::Cnc, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xd4, &mut res, v)?
            }
            Instruction(InstructionCode::Sui, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xd6, &mut res, v)?
            }
            Instruction(InstructionCode::Rc, _, _) => res.push(StageOneValue::Word(0xd8)),
            Instruction(InstructionCode::Jc, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xda, &mut res, v)?
            }
            Instruction(InstructionCode::In, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xdb, &mut res, v)?
            }
            Instruction(InstructionCode::Cc, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xdc, &mut res, v)?
            }
            Instruction(InstructionCode::Sbi, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xde, &mut res, v)?
            }
            Instruction(InstructionCode::Rpo, _, _) => res.push(StageOneValue::Word(0xe0)),
            Instruction(InstructionCode::Jpo, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xe2, &mut res, v)?
            }
            Instruction(InstructionCode::Xthl, _, _) => res.push(StageOneValue::Word(0xe3)),
            Instruction(InstructionCode::Cpo, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xe4, &mut res, v)?
            }
            Instruction(InstructionCode::Ani, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xe6, &mut res, v)?
            }
            Instruction(InstructionCode::Rpe, _, _) => res.push(StageOneValue::Word(0xe8)),
            Instruction(InstructionCode::Pchl, _, _) => res.push(StageOneValue::Word(0xe9)),
            Instruction(InstructionCode::Jpe, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xea, &mut res, v)?
            }
            Instruction(InstructionCode::Xchg, _, _) => res.push(StageOneValue::Word(0xeb)),
            Instruction(InstructionCode::Cpe, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xec, &mut res, v)?
            }
            Instruction(InstructionCode::Xri, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xee, &mut res, v)?
            }
            Instruction(InstructionCode::Rp, _, _) => res.push(StageOneValue::Word(0xf0)),
            Instruction(InstructionCode::Jp, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xf2, &mut res, v)?
            }
            Instruction(InstructionCode::Di, _, _) => res.push(StageOneValue::Word(0xf3)),
            Instruction(InstructionCode::Cp, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xf4, &mut res, v)?
            }
            Instruction(InstructionCode::Ori, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xf6, &mut res, v)?
            }
            Instruction(InstructionCode::Rm, _, _) => res.push(StageOneValue::Word(0xf8)),
            Instruction(InstructionCode::Sphl, _, _) => res.push(StageOneValue::Word(0xf9)),
            Instruction(InstructionCode::Jm, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xfa, &mut res, v)?
            }
            Instruction(InstructionCode::Ei, _, _) => res.push(StageOneValue::Word(0xfb)),
            Instruction(InstructionCode::Cm, Some(InstructionArgument::TwoWord(v)), _) => {
                self.add_simple_two_word_instruction(0xfc, &mut res, v)?
            }
            Instruction(InstructionCode::Cpi, Some(InstructionArgument::Word(v)), _) => {
                self.add_simple_word_instruction(0xfe, &mut res, v)?
            }
            i => panic!("unfined method {:?}", i),
        };
        Ok(res)
    }
}
