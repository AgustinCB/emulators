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
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Adc,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Adc { source: l })),
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Add,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Add { source: l })),
            (InstructionCode::Aci, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Aci { byte })),
            (InstructionCode::Adi, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Adi { byte })),
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Ana,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ana { source: l })),
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
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Cmp,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cmp { source: l })),
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
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Dcr,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dcr { source: l })),
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
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Inr,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Inr { source: l })),
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
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Mov,
                &Some(AssemblerToken::DataStore(d@Location::Memory))) => {
                self.source.next();
                match self.source.peek() {
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::A
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::B
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::C
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::D
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::E
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::H
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Register {
                        register: RegisterType::L
                    })) |
                    Some(&AssemblerToken::DataStore(s@Location::Memory)) => {
                        self.source.next();
                        Ok(Expression::Instruction(Intel8080Instruction::Mov {
                            destiny: d,
                            source: s,
                        }))
                    },
                    _ => Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Noop, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Noop)),
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Ora,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ora { source: l })),
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
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Sbb,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sbb { source: l })),
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
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Sub,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sub { source: l })),
            (InstructionCode::Sui, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sui { byte })),
            (InstructionCode::Xchg, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xchg)),
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Xra,
                &Some(AssemblerToken::DataStore(l@Location::Memory))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xra { source: l })),
            (InstructionCode::Xri, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xri { byte })),
            (InstructionCode::Xthl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xthl)),
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }
    }
}