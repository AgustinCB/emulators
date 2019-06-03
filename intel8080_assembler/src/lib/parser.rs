extern crate failure;
extern crate intel8080cpu;

use super::*;
use failure::Error;
use intel8080cpu::{Location, RegisterType};
use std::iter::{IntoIterator, Peekable};
use std::vec::IntoIter;

pub struct Parser {
    source: Peekable<IntoIter<AssemblerToken>>,
    expressions: Vec<Statement>,
}

impl Parser {
    pub fn new(source: Vec<AssemblerToken>) -> Parser {
        Parser {
            source: source.into_iter().peekable(),
            expressions: Vec::new(),
        }
    }

    pub fn parse_statements(mut self) -> Result<Vec<Statement>, Error> {
        while let Some(input) = self.source.next() {
            self.parse_statement(&input)?;
        }
        Ok(self.expressions)
    }

    fn parse_statement(&mut self, input: &AssemblerToken) -> Result<(), Error> {
        let next = self.source.peek().map(|a| (*a).clone());
        let expression = match (input, next) {
            (
                AssemblerToken {
                    token_type: AssemblerTokenType::Org,
                    ..
                },
                Some(AssemblerToken {
                    token_type: AssemblerTokenType::TwoWord(n),
                    ..
                }),
            ) => {
                self.source.next();
                Ok(Statement::OrgStatement(n))
            }
            (
                AssemblerToken {
                    token_type: AssemblerTokenType::Org,
                    line,
                },
                ref got,
            ) => Err(Error::from(AssemblerError::ExpectingNumber {
                got: got.clone().map(|v| v.token_type),
                line: *line,
            })),
            (
                AssemblerToken {
                    token_type: AssemblerTokenType::LabelToken(ref label),
                    ..
                },
                Some(AssemblerToken {
                    token_type: AssemblerTokenType::Colon,
                    ..
                }),
            ) => {
                self.source.next();
                Ok(Statement::LabelDefinitionStatement(label.clone()))
            }
            (
                AssemblerToken {
                    token_type: AssemblerTokenType::LabelToken(ref label),
                    ..
                },
                Some(AssemblerToken {
                    token_type: AssemblerTokenType::Dw,
                    line,
                }),
            ) => self.parse_two_word_definition(label, line),

            (
                AssemblerToken {
                    token_type: AssemblerTokenType::LabelToken(ref label),
                    ..
                },
                Some(AssemblerToken {
                    token_type: AssemblerTokenType::Db,
                    line,
                }),
            ) => self.parse_word_definition(label, line),
            (
                AssemblerToken {
                    token_type: AssemblerTokenType::InstructionCode(instruction),
                    line,
                },
                ref next,
            ) => self.parse_instruction(
                instruction,
                &next.clone().map(|t| t.token_type),
                line.clone(),
            ),
            (t, _) => Err(Error::from(AssemblerError::UndefinedError { line: t.line })),
        }?;
        self.expressions.push(expression);
        Ok(())
    }

    fn parse_word_definition(
        &mut self,
        label: &LabelExpression,
        line: usize,
    ) -> Result<Statement, Error> {
        self.source.next();
        let op = self.parse_operation(line)?;
        Ok(Statement::WordDefinitionStatement(label.clone(), op))
    }

    fn parse_two_word_definition(
        &mut self,
        label: &LabelExpression,
        line: usize,
    ) -> Result<Statement, Error> {
        self.source.next();
        let op = self.parse_operation(line)?;
        Ok(Statement::TwoWordDefinitionStatement(label.clone(), op))
    }

    fn parse_operation(&mut self, line: usize) -> Result<OperationExpression, Error> {
        let left_side = self.parse_and_operation(line)?;
        let next = self.source.peek().cloned();
        match next {
            Some(AssemblerToken {
                token_type: AssemblerTokenType::Or,
                line,
            }) => {
                self.source.next();
                let right_side = self.parse_operation(line)?;
                Ok(OperationExpression::Or(
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            Some(AssemblerToken {
                token_type: AssemblerTokenType::Xor,
                line,
            }) => {
                self.source.next();
                let right_side = self.parse_operation(line)?;
                Ok(OperationExpression::Xor(
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            _ => Ok(left_side),
        }
    }

    fn parse_and_operation(&mut self, line: usize) -> Result<OperationExpression, Error> {
        let left_side = self.parse_not_operation(line)?;
        let next = self.source.peek().map(|t| t.token_type.clone());
        match next {
            Some(AssemblerTokenType::And) => {
                self.source.next();
                let right_side = self.parse_and_operation(line)?;
                Ok(OperationExpression::And(
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            _ => Ok(left_side),
        }
    }

    fn parse_not_operation(&mut self, line: usize) -> Result<OperationExpression, Error> {
        let next = self.source.peek().cloned();
        match next {
            Some(AssemblerToken {
                token_type: AssemblerTokenType::Not,
                line,
            }) => {
                self.source.next();
                let right_side = self.parse_sum_operations(line)?;
                Ok(OperationExpression::Not(Box::new(right_side)))
            }
            Some(AssemblerToken {
                line,
                ..
            }) => self.parse_sum_operations(line),
            None => Err(Error::from(AssemblerError::UnexpectedEndOfExpression {
                line,
            })),
        }
    }

    fn parse_sum_operations(&mut self, line: usize) -> Result<OperationExpression, Error> {
        let left_side = self.parse_last_operations(line)?;
        let next = self.source.peek().map(|t| t.token_type.clone());
        match next {
            Some(AssemblerTokenType::Plus) => {
                self.source.next();
                let right_side = self.parse_sum_operations(line)?;
                Ok(OperationExpression::Sum(
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            Some(AssemblerTokenType::Minus) => {
                self.source.next();
                let right_side = self.parse_sum_operations(line)?;
                Ok(OperationExpression::Sub(
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            _ => Ok(left_side),
        }
    }

    fn parse_last_operations(&mut self, line: usize) -> Result<OperationExpression, Error> {
        let op = self.parse_group(line)?;
        let next = self.source.peek().map(|t| t.token_type.clone());
        match next {
            Some(AssemblerTokenType::Div) => {
                self.source.next();
                let right_side = self.parse_last_operations(line)?;
                Ok(OperationExpression::Div(Box::new(op), Box::new(right_side)))
            }
            Some(AssemblerTokenType::Mod) => {
                self.source.next();
                let right_side = self.parse_last_operations(line)?;
                Ok(OperationExpression::Mod(Box::new(op), Box::new(right_side)))
            }
            Some(AssemblerTokenType::Mult) => {
                self.source.next();
                let right_side = self.parse_last_operations(line)?;
                Ok(OperationExpression::Mult(
                    Box::new(op),
                    Box::new(right_side),
                ))
            }
            Some(AssemblerTokenType::Shl) => {
                self.source.next();
                let right_side = self.parse_last_operations(line)?;
                Ok(OperationExpression::Shl(Box::new(op), Box::new(right_side)))
            }
            Some(AssemblerTokenType::Shr) => {
                self.source.next();
                let right_side = self.parse_last_operations(line)?;
                Ok(OperationExpression::Shr(Box::new(op), Box::new(right_side)))
            }
            _ => Ok(op),
        }
    }

    fn parse_group(&mut self, line: usize) -> Result<OperationExpression, Error> {
        let next = self.source.peek().cloned();
        match next {
            Some(AssemblerToken {
                token_type: AssemblerTokenType::LeftParen,
                line,
            }) => {
                self.source.next();
                let op = self.parse_operation(line)?;
                self.consume(AssemblerTokenType::RightParen, line)?;
                Ok(OperationExpression::Group(Box::new(op)))
            }
            Some(AssemblerToken {
                line,
                ..
            }) => {
                let word = self.parse_two_word(line)?;
                Ok(OperationExpression::Operand(word))
            }
            None => Err(Error::from(AssemblerError::UnexpectedEndOfExpression {
                line,
            })),
        }
    }

    fn parse_two_word(&mut self, line: usize) -> Result<TwoWordExpression, Error> {
        let next = self.source.peek().map(|t| t.token_type.clone());
        let res = match next {
            Some(AssemblerTokenType::Char(c_value)) => Ok(TwoWordExpression::Char(c_value)),
            Some(AssemblerTokenType::Dollar) => Ok(TwoWordExpression::Dollar),
            Some(AssemblerTokenType::TwoWord(value)) => Ok(TwoWordExpression::Literal(value)),
            Some(AssemblerTokenType::LabelToken(label)) => Ok(TwoWordExpression::Label(label)),
            got => Err(Error::from(AssemblerError::ExpectingNumber { got, line })),
        }?;
        self.source.next();
        Ok(res)
    }

    fn parse_instruction(
        &mut self,
        instruction: &InstructionCode,
        next: &Option<AssemblerTokenType>,
        line: usize,
    ) -> Result<Statement, Error> {
        match (instruction, next) {
            (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Adc,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Adc, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Adc)
            }
            (InstructionCode::Adc, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Add,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Add, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Add)
            }
            (InstructionCode::Add, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Aci, _) => self.parse_word_instruction(InstructionCode::Aci, line),
            (InstructionCode::Adi, _) => self.parse_word_instruction(InstructionCode::Adi, line),
            (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Ana,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Ana, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Ana)
            }
            (InstructionCode::Ana, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Ani, _) => self.parse_word_instruction(InstructionCode::Ani, line),
            (InstructionCode::Call, _) => {
                self.parse_two_word_instruction(InstructionCode::Call, line)
            }
            (InstructionCode::Cc, _) => self.parse_two_word_instruction(InstructionCode::Cc, line),
            (InstructionCode::Cm, _) => self.parse_two_word_instruction(InstructionCode::Cm, line),
            (InstructionCode::Cma, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Cma,
                None,
                None,
            ))),
            (InstructionCode::Cmc, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Cmc,
                None,
                None,
            ))),
            (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Cmp,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Cmp, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Cmp)
            }
            (InstructionCode::Cmp, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Cnc, _) => {
                self.parse_two_word_instruction(InstructionCode::Cnc, line)
            }
            (InstructionCode::Cnz, _) => {
                self.parse_two_word_instruction(InstructionCode::Cnz, line)
            }
            (InstructionCode::Cp, _) => self.parse_two_word_instruction(InstructionCode::Cp, line),
            (InstructionCode::Cpe, _) => {
                self.parse_two_word_instruction(InstructionCode::Cpe, line)
            }
            (InstructionCode::Cpi, _) => self.parse_word_instruction(InstructionCode::Cpi, line),
            (InstructionCode::Cpo, _) => {
                self.parse_two_word_instruction(InstructionCode::Cpo, line)
            }
            (InstructionCode::Cz, _) => self.parse_two_word_instruction(InstructionCode::Cz, line),
            (InstructionCode::Daa, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Daa,
                None,
                None,
            ))),
            (
                InstructionCode::Dad,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Dad,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Dad,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Dad,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::Sp,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Dad),
            (InstructionCode::Dad, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Dcr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Dcr, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Dcr)
            }
            (InstructionCode::Dcr, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Dcx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Dcx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Dcx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Dcx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::Sp,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Dcx),
            (InstructionCode::Dcx, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Di, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Di,
                None,
                None,
            ))),
            (InstructionCode::Ei, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Ei,
                None,
                None,
            ))),
            (InstructionCode::Hlt, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Hlt,
                None,
                None,
            ))),
            (InstructionCode::In, _) => self.parse_word_instruction(InstructionCode::In, line),
            (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Inr,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Inr, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Inr)
            }
            (InstructionCode::Inr, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Inx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Inx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Inx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Inx,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::Sp,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Inx),
            (InstructionCode::Inx, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Jc, _) => self.parse_two_word_instruction(InstructionCode::Jc, line),
            (InstructionCode::Jm, _) => self.parse_two_word_instruction(InstructionCode::Jm, line),
            (InstructionCode::Jmp, _) => {
                self.parse_two_word_instruction(InstructionCode::Jmp, line)
            }
            (InstructionCode::Jnc, _) => {
                self.parse_two_word_instruction(InstructionCode::Jnc, line)
            }
            (InstructionCode::Jnz, _) => {
                self.parse_two_word_instruction(InstructionCode::Jnz, line)
            }
            (InstructionCode::Jp, _) => self.parse_two_word_instruction(InstructionCode::Jp, line),
            (InstructionCode::Jpe, _) => {
                self.parse_two_word_instruction(InstructionCode::Jpe, line)
            }
            (InstructionCode::Jpo, _) => {
                self.parse_two_word_instruction(InstructionCode::Jpo, line)
            }
            (InstructionCode::Jz, _) => self.parse_two_word_instruction(InstructionCode::Jz, line),
            (InstructionCode::Lda, _) => {
                self.parse_two_word_instruction(InstructionCode::Lda, line)
            }
            (
                InstructionCode::Ldax,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Ldax,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Ldax),
            (InstructionCode::Ldax, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Lhld, _) => {
                self.parse_two_word_instruction(InstructionCode::Lhld, line)
            }
            (
                InstructionCode::Lxi,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Lxi,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Lxi,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Lxi,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::Sp,
                    },
                )),
            ) => {
                self.source.next();
                self.consume(AssemblerTokenType::Comma, line)?;
                let op = self.parse_operation(line)?;
                Ok(Statement::InstructionExprStmt(Instruction(
                    InstructionCode::Lxi,
                    Some(InstructionArgument::DataStore(l)),
                    Some(InstructionArgument::TwoWord(op)),
                )))
            }
            (InstructionCode::Lxi, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Mov,
                &Some(AssemblerTokenType::DataStore(
                    d @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Mov, &Some(AssemblerTokenType::DataStore(d @ Location::Memory))) => {
                self.source.next();
                self.consume(AssemblerTokenType::Comma, line)?;
                match self.source.peek().map(|v| v.clone().token_type) {
                    Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::A,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::B,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::C,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::D,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::E,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::H,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(
                        s @ Location::Register {
                            register: RegisterType::L,
                        },
                    ))
                    | Some(AssemblerTokenType::DataStore(s @ Location::Memory)) => {
                        self.source.next();
                        Ok(Statement::InstructionExprStmt(Instruction(
                            InstructionCode::Mov,
                            Some(InstructionArgument::DataStore(d)),
                            Some(InstructionArgument::DataStore(s)),
                        )))
                    }
                    _ => Err(Error::from(AssemblerError::InvalidInstructionArgument {
                        line,
                    })),
                }
            }
            (InstructionCode::Mov, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Mvi,
                &Some(AssemblerTokenType::DataStore(
                    s @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Mvi, &Some(AssemblerTokenType::DataStore(s @ Location::Memory))) => {
                self.source.next();
                self.consume(AssemblerTokenType::Comma, line)?;
                let op = self.parse_operation(line)?;
                Ok(Statement::InstructionExprStmt(Instruction(
                    InstructionCode::Mvi,
                    Some(InstructionArgument::DataStore(s)),
                    Some(InstructionArgument::from(op)),
                )))
            }
            (InstructionCode::Mvi, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Noop, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Noop,
                None,
                None,
            ))),
            (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Ora,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Ora, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Ora)
            }
            (InstructionCode::Ora, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Ori, _) => self.parse_word_instruction(InstructionCode::Ori, line),
            (InstructionCode::Out, _) => self.parse_word_instruction(InstructionCode::Out, line),
            (InstructionCode::Pchl, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Pchl,
                None,
                None,
            ))),
            (
                InstructionCode::Pop,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Pop,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Pop,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Pop,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::Psw,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Pop),
            (InstructionCode::Pop, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (
                InstructionCode::Push,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Push,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Push,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Push,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::Psw,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Push),
            (InstructionCode::Push, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Ral, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Ral,
                None,
                None,
            ))),
            (InstructionCode::Rar, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rar,
                None,
                None,
            ))),
            (InstructionCode::Rc, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rc,
                None,
                None,
            ))),
            (InstructionCode::Ret, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Ret,
                None,
                None,
            ))),
            (InstructionCode::Rlc, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rlc,
                None,
                None,
            ))),
            (InstructionCode::Rm, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rm,
                None,
                None,
            ))),
            (InstructionCode::Rnc, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rnc,
                None,
                None,
            ))),
            (InstructionCode::Rnz, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rnz,
                None,
                None,
            ))),
            (InstructionCode::Rp, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rp,
                None,
                None,
            ))),
            (InstructionCode::Rpe, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rpe,
                None,
                None,
            ))),
            (InstructionCode::Rpo, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rpo,
                None,
                None,
            ))),
            (InstructionCode::Rrc, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rrc,
                None,
                None,
            ))),
            (InstructionCode::Rst, _) => self.parse_word_instruction(InstructionCode::Rst, line),
            (InstructionCode::Rz, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Rz,
                None,
                None,
            ))),
            (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Sbb,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Sbb, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Sbb)
            }
            (InstructionCode::Sbb, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Sbi, _) => self.parse_word_instruction(InstructionCode::Sbi, line),
            (InstructionCode::Shld, _) => {
                self.parse_two_word_instruction(InstructionCode::Shld, line)
            }
            (InstructionCode::Sphl, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Sphl,
                None,
                None,
            ))),
            (InstructionCode::Sta, _) => {
                self.parse_two_word_instruction(InstructionCode::Sta, line)
            }
            (
                InstructionCode::Stax,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Stax,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            ) => self.parse_instruction_with_location(l, InstructionCode::Stax),
            (InstructionCode::Stax, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Stc, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Stc,
                None,
                None,
            ))),
            (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Sub,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Sub, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Sub)
            }
            (InstructionCode::Sub, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Sui, _) => self.parse_word_instruction(InstructionCode::Sui, line),
            (InstructionCode::Xchg, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Xchg,
                None,
                None,
            ))),
            (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::A,
                    },
                )),
            )
            | (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::B,
                    },
                )),
            )
            | (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::C,
                    },
                )),
            )
            | (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::D,
                    },
                )),
            )
            | (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::E,
                    },
                )),
            )
            | (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::H,
                    },
                )),
            )
            | (
                InstructionCode::Xra,
                &Some(AssemblerTokenType::DataStore(
                    l @ Location::Register {
                        register: RegisterType::L,
                    },
                )),
            )
            | (InstructionCode::Xra, &Some(AssemblerTokenType::DataStore(l @ Location::Memory))) => {
                self.parse_instruction_with_location(l, InstructionCode::Xra)
            }
            (InstructionCode::Xra, _) => {
                Err(Error::from(AssemblerError::InvalidInstructionArgument {
                    line,
                }))
            }
            (InstructionCode::Xri, _) => self.parse_word_instruction(InstructionCode::Xri, line),
            (InstructionCode::Xthl, _) => Ok(Statement::InstructionExprStmt(Instruction(
                InstructionCode::Xthl,
                None,
                None,
            ))),
        }
    }

    fn consume(&mut self, token: AssemblerTokenType, line: usize) -> Result<(), Error> {
        match self.source.next() {
            Some(AssemblerToken {
                ref token_type,
                ..
            }) if token_type == &token => Ok(()),
            Some(AssemblerToken { token_type, line }) => {
                Err(Error::from(AssemblerError::ExpectingToken {
                    expected: token,
                    got: Some(token_type),
                    line,
                }))
            }
            None => Err(Error::from(AssemblerError::ExpectingToken {
                expected: token,
                got: None,
                line,
            })),
        }
    }

    #[inline]
    fn parse_instruction_with_location(
        &mut self,
        l: Location,
        i: InstructionCode,
    ) -> Result<Statement, Error> {
        self.source.next();
        Ok(Statement::InstructionExprStmt(Instruction(
            i,
            Some(InstructionArgument::DataStore(l)),
            None,
        )))
    }

    #[inline]
    fn parse_word_instruction(
        &mut self,
        i: InstructionCode,
        line: usize,
    ) -> Result<Statement, Error> {
        let op = self.parse_operation(line)?;
        Ok(Statement::InstructionExprStmt(Instruction(
            i.clone(),
            Some(InstructionArgument::Word(op)),
            None,
        )))
    }

    #[inline]
    fn parse_two_word_instruction(
        &mut self,
        i: InstructionCode,
        line: usize,
    ) -> Result<Statement, Error> {
        let op = self.parse_operation(line)?;
        Ok(Statement::InstructionExprStmt(Instruction(
            i.clone(),
            Some(InstructionArgument::TwoWord(op)),
            None,
        )))
    }
}
