extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Location, Intel8080Instruction, RegisterType};
use std::iter::{IntoIterator, Peekable};
use std::vec::IntoIter;
use super::{AssemblerError, AssemblerToken, Expression, InstructionCode};

pub struct Parser {
    source: Peekable<IntoIter<AssemblerToken>>,
    expressions: Vec<Expression>,
}

impl Parser {
    pub fn new(source: Vec<AssemblerToken>) -> Parser {
        Parser {
            source: source.into_iter().peekable(),
            expressions: Vec::new(),
        }
    }

    pub fn parse_expressions(&mut self) -> Result<(), Error> {
        while let Some(input) = self.source.next() {
            self.parse_expression(&input)?;
        }
        Ok(())
    }

    fn parse_expression(&mut self, input: &AssemblerToken) -> Result<(), Error> {
        let next = self.source.peek().map(|a| (*a).clone());
        let expression = match (input, next) {
            (AssemblerToken::LabelToken(label), Some(AssemblerToken::Colon)) =>
                Ok(Expression::LabelDefinition((*label).clone())),
            (AssemblerToken::LabelToken(label), Some(AssemblerToken::Dw)) => {
                self.source.next();
                if let Some(AssemblerToken::Word(value)) = self.source.peek() {
                    Ok(Expression::WordDefinition { value: *value, label: (*label).clone() })
                } else {
                    Err(Error::from(AssemblerError::ExpectingNumber))
                }
            },
            (AssemblerToken::LabelToken(label), Some(AssemblerToken::Db)) => {
                self.source.next();
                if let Some(AssemblerToken::Byte(value)) = self.source.peek() {
                    Ok(Expression::ByteDefinition { value: *value, label: (*label).clone() })
                } else {
                    Err(Error::from(AssemblerError::ExpectingNumber))
                }
            },
            (AssemblerToken::InstructionCode(instruction), ref next) =>
                self.parse_instruction(instruction, next),
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }?;
        self.expressions.push(expression);
        Ok(())
    }

    fn parse_instruction(&mut self, instruction: &InstructionCode, next: &Option<AssemblerToken>)
        -> Result<Expression, Error> {
        match (instruction, next) {
            (InstructionCode::Aci, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Aci { byte })),
            (InstructionCode::Adi, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Adi { byte })),
            (InstructionCode::Ani, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ani { byte })),
            (InstructionCode::Call, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Call {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cc, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cm, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cm {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cma, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cma)),
            (InstructionCode::Cmc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cmc)),
            (InstructionCode::Cpi, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cpi { byte })),
            (InstructionCode::Cnc, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cnc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cnz, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cnz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cp, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cp {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cpe, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cpe {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cpo, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cpo {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cz, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Cz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Daa, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Daa)),
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dad { register: r })),
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dcx { register: r })),
            (InstructionCode::Di, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Di)),
            (InstructionCode::Ei, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ei)),
            (InstructionCode::Jc, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jm, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jm {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jmp, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jmp {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jnc, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jnc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jnz, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jnz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jp, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jp {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jpe, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jpe {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jpo, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jpo {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jz, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Jz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Hlt, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Hlt)),
            (InstructionCode::In, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::In { byte })),
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Inx { register: r })),
            (InstructionCode::Lhld, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Lhld {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Lda, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Lda {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Ldax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Ldax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ldax { register: r })),
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp}))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(low_byte)) = self.source.peek() {
                    self.source.next();
                    if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                        Ok(Expression::Instruction(Intel8080Instruction::Lxi {
                            register: r,
                            high_byte,
                            low_byte,
                        }))
                    } else {
                        Err(Error::from(AssemblerError::InvalidInstructionArgument))
                    }
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Noop, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Noop)),
            (InstructionCode::Ori, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ori { byte })),
            (InstructionCode::Out, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Out { byte })),
            (InstructionCode::Pchl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pchl)),
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Psw }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pop { register: r })),
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Psw }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Push { register: r })),
            (InstructionCode::Ral, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ral)),
            (InstructionCode::Rar, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rar)),
            (InstructionCode::Rc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rc)),
            (InstructionCode::Ret, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ret)),
            (InstructionCode::Rlc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rlc)),
            (InstructionCode::Rm, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rm)),
            (InstructionCode::Rnc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rnc)),
            (InstructionCode::Rnz, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rnz)),
            (InstructionCode::Rp, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rp)),
            (InstructionCode::Rpe, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rpe)),
            (InstructionCode::Rpo, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rpo)),
            (InstructionCode::Rrc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rrc)),
            (InstructionCode::Rst, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rst { byte })),
            (InstructionCode::Rz, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rz)),
            (InstructionCode::Sbi, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sbi { byte })),
            (InstructionCode::Shld, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Shld {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Sphl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sphl)),
            (InstructionCode::Sta, &Some(AssemblerToken::Byte(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Byte(high_byte)) = self.source.peek() {
                    Ok(Expression::Instruction(Intel8080Instruction::Sta {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Stax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Stax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Stax { register: r })),
            (InstructionCode::Stc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Stc)),
            (InstructionCode::Sui, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sui { byte })),
            (InstructionCode::Xchg, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xchg)),
            (InstructionCode::Xri, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xri { byte })),
            (InstructionCode::Xthl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xthl)),
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }
    }
}