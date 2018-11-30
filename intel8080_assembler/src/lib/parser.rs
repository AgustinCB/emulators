extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Location, RegisterType};
use std::iter::{IntoIterator, Peekable};
use std::vec::IntoIter;
use super::*;

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
            (AssemblerToken::Org, _) => {
                self.source.next();
                let op = self.parse_operation()?;
                Ok(Statement::OrgStatement(op))
            },
            (&AssemblerToken::LabelToken(ref label), Some(AssemblerToken::Colon)) => {
                self.source.next();
                Ok(Statement::LabelDefinitionStatement(label.clone()))
            },
            (&AssemblerToken::LabelToken(ref label), Some(AssemblerToken::Dw)) =>
                self.parse_two_word_definition(label),
            (&AssemblerToken::LabelToken(ref label), Some(AssemblerToken::Db)) =>
                self.parse_word_definition(label),
            (AssemblerToken::InstructionCode(instruction), ref next) =>
                self.parse_instruction(instruction, next),
            (_, next@_) => {
                eprintln!("INPUT {:?}, NEXT {:?}", input, next);
                Err(Error::from(AssemblerError::UndefinedError))
            },
        }?;
        self.expressions.push(expression);
        Ok(())
    }

    fn parse_word_definition(&mut self, label: &LabelExpression) -> Result<Statement, Error> {
        self.source.next();
        let op = self.parse_operation()?;
        Ok(Statement::WordDefinitionStatement(label.clone(), op))
    }

    fn parse_two_word_definition(&mut self, label: &LabelExpression) -> Result<Statement, Error> {
        self.source.next();
        let op = self.parse_operation()?;
        Ok(Statement::TwoWordDefinitionStatement(label.clone(), op))
    }

    fn parse_operation(&mut self) -> Result<OperationExpression, Error> {
        let left_side = self.parse_and_operation()?;
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(AssemblerToken::Or) => {
                let right_side = self.parse_operation()?;
                Ok(OperationExpression::Or(Box::new(left_side), Box::new(right_side)))
            },
            Some(AssemblerToken::Xor) => {
                let right_side = self.parse_operation()?;
                Ok(OperationExpression::Xor(Box::new(left_side), Box::new(right_side)))
            },
            _ => Ok(left_side),
        }
    }

    fn parse_and_operation(&mut self) -> Result<OperationExpression, Error> {
        let left_side = self.parse_not_operation()?;
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(AssemblerToken::And) => {
                let right_side = self.parse_and_operation()?;
                Ok(OperationExpression::And(Box::new(left_side), Box::new(right_side)))
            }
            _ => Ok(left_side)
        }
    }

    fn parse_not_operation(&mut self) -> Result<OperationExpression, Error> {
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(AssemblerToken::Not) => {
                let right_side = self.parse_sum_operations()?;
                Ok(OperationExpression::Not(Box::new(right_side)))
            },
            _ => self.parse_sum_operations(),
        }
    }

    fn parse_sum_operations(&mut self) -> Result<OperationExpression, Error> {
        let left_side = self.parse_last_operations()?;
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(AssemblerToken::Plus) => {
                self.source.next();
                let right_side = self.parse_sum_operations()?;
                Ok(OperationExpression::Sum(Box::new(left_side), Box::new(right_side)))
            },
            Some(AssemblerToken::Minus) => {
                self.source.next();
                let right_side = self.parse_sum_operations()?;
                Ok(OperationExpression::Sub(Box::new(left_side), Box::new(right_side)))
            },
            Some(_) => Ok(left_side),
            None => Err(Error::from(AssemblerError::ExpectingOperation { got: None })),
        }
    }

    fn parse_last_operations(&mut self) -> Result<OperationExpression, Error> {
        let two_word = self.parse_two_word()?;
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(AssemblerToken::Div) => {
                self.source.next();
                let right_side = self.parse_last_operations()?;
                Ok(OperationExpression::Div(two_word, Box::new(right_side)))
            },
            Some(AssemblerToken::Mod) => {
                self.source.next();
                let right_side = self.parse_last_operations()?;
                Ok(OperationExpression::Mod(two_word, Box::new(right_side)))
            },
            Some(AssemblerToken::Mult) => {
                self.source.next();
                let right_side = self.parse_last_operations()?;
                Ok(OperationExpression::Mult(two_word, Box::new(right_side)))
            },
            Some(AssemblerToken::Shl) => {
                self.source.next();
                let right_side = self.parse_last_operations()?;
                Ok(OperationExpression::Shl(two_word, Box::new(right_side)))
            },
            Some(AssemblerToken::Shr) => {
                self.source.next();
                let right_side = self.parse_last_operations()?;
                Ok(OperationExpression::Shr(two_word, Box::new(right_side)))
            },
            Some(_) => Ok(OperationExpression::Operand(two_word)),
            None => Err(Error::from(AssemblerError::ExpectingOperation { got: None })),
        }
    }

    fn parse_two_word(&mut self) -> Result<TwoWordExpression, Error> {
        let next = self.source.peek().map(|t| (*t).clone());
        let res = match next {
            Some(AssemblerToken::Char(c_value)) => Ok(TwoWordExpression::Char(c_value)),
            Some(AssemblerToken::Dollar) => Ok(TwoWordExpression::Dollar),
            Some(AssemblerToken::TwoWord(value)) => Ok(TwoWordExpression::Literal(value)),
            Some(AssemblerToken::LabelToken(label)) => Ok(TwoWordExpression::Label(label)),
            got => Err(Error::from(AssemblerError::ExpectingNumber { got }))
        };
        res.iter().for_each(|_| { self.source.next(); });
        res
    }

    fn consume_comma(&mut self) -> Result<(), Error> {
        match self.source.next() {
            Some(AssemblerToken::Comma) => Ok(()),
            Some(token) => Err(Error::from(AssemblerError::ExpectingToken {
                expected: AssemblerToken::Comma,
                got: Some(token),
            })),
            None => Err(Error::from(AssemblerError::ExpectingToken {
                expected: AssemblerToken::Comma,
                got: None,
            })),
        }
    }

    fn parse_instruction(&mut self, instruction: &InstructionCode, next: &Option<AssemblerToken>)
        -> Result<Statement, Error> {
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
                self.parse_instruction_with_location(l, InstructionCode::Adc),
            (InstructionCode::Adc, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
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
                self.parse_instruction_with_location(l, InstructionCode::Add),
            (InstructionCode::Add, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Aci, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Aci),
            (InstructionCode::Aci, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Adi, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Adi),
            (InstructionCode::Adi, n) =>
                Err(Error::from(AssemblerError::ExpectingNumber { got: (*n).clone() })),
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
                self.parse_instruction_with_location(l, InstructionCode::Ana),
            (InstructionCode::Ana, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ani, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Ani),
            (InstructionCode::Ani, n) =>
                Err(Error::from(AssemblerError::ExpectingNumber { got: (*n).clone() })),
            (InstructionCode::Call, _) => self.parse_two_word_instruction(InstructionCode::Call),
            (InstructionCode::Cc, _) => self.parse_two_word_instruction(InstructionCode::Cc),
            (InstructionCode::Cm, _) => self.parse_two_word_instruction(InstructionCode::Cm),
            (InstructionCode::Cma, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Cma, None, None))),
            (InstructionCode::Cmc, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Cmc, None, None))),
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
                self.parse_instruction_with_location(l, InstructionCode::Cmp),
            (InstructionCode::Cmp, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cpi, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Cpi),
            (InstructionCode::Cpi, n) =>
                Err(Error::from(AssemblerError::ExpectingNumber { got: (*n).clone() })),
            (InstructionCode::Cnc, _) => self.parse_two_word_instruction(InstructionCode::Cnc),
            (InstructionCode::Cnz, _) => self.parse_two_word_instruction(InstructionCode::Cnz),
            (InstructionCode::Cp, _) => self.parse_two_word_instruction(InstructionCode::Cp),
            (InstructionCode::Cpe, _) => self.parse_two_word_instruction(InstructionCode::Cpe),
            (InstructionCode::Cpo, _) => self.parse_two_word_instruction(InstructionCode::Cpo),
            (InstructionCode::Cz, _) => self.parse_two_word_instruction(InstructionCode::Cz),
            (InstructionCode::Daa, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Daa, None, None))),
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::Sp }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Dad),
            (InstructionCode::Dad, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
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
                self.parse_instruction_with_location(l, InstructionCode::Dcr),
            (InstructionCode::Dcr, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::Sp }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Dcx),
            (InstructionCode::Dcx, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Di, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Di, None, None))),
            (InstructionCode::Ei, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Ei, None, None))),
            (InstructionCode::Hlt, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Hlt, None, None))),
            (InstructionCode::In, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::In),
            (InstructionCode::In, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
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
                self.parse_instruction_with_location(l, InstructionCode::Inr),
            (InstructionCode::Inr, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::Sp }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Inx),
            (InstructionCode::Inx, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jc, _) => self.parse_two_word_instruction(InstructionCode::Jc),
            (InstructionCode::Jm, _) => self.parse_two_word_instruction(InstructionCode::Jm),
            (InstructionCode::Jmp, _) => self.parse_two_word_instruction(InstructionCode::Jmp),
            (InstructionCode::Jnc, _) => self.parse_two_word_instruction(InstructionCode::Jnc),
            (InstructionCode::Jnz, _) => self.parse_two_word_instruction(InstructionCode::Jnz),
            (InstructionCode::Jp, _) => self.parse_two_word_instruction(InstructionCode::Jp),
            (InstructionCode::Jpe, _) => self.parse_two_word_instruction(InstructionCode::Jpe),
            (InstructionCode::Jpo, _) => self.parse_two_word_instruction(InstructionCode::Jpo),
            (InstructionCode::Jz, _) => self.parse_two_word_instruction(InstructionCode::Jz),
            (InstructionCode::Lda, _) => self.parse_two_word_instruction(InstructionCode::Lda),
            (InstructionCode::Ldax,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Ldax,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Ldax),
            (InstructionCode::Ldax, _) => Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Lhld, _) => self.parse_two_word_instruction(InstructionCode::Lhld),
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::Sp}))) => {
                self.source.next();
                self.consume_comma()?;
                let op = self.parse_operation()?;
                Ok(Statement::InstructionExprStmt(Instruction(
                    InstructionCode::Lxi,
                    Some(InstructionArgument::DataStore(l)),
                    Some(InstructionArgument::TwoWord(op))
                )))
            },
            (InstructionCode::Lxi, n) => Err(Error::from(AssemblerError::ExpectingNumber {
                got: (*n).clone(),
            })),
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
                self.consume_comma()?;
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
                        Ok(Statement::InstructionExprStmt(Instruction(
                            InstructionCode::Mov,
                            Some(InstructionArgument::DataStore(d)),
                            Some(InstructionArgument::DataStore(s)),
                        )))
                    },
                    _ => Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Mov, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::A }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::C }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::E }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Register { register: RegisterType::L }))) |
            (InstructionCode::Mvi,
                &Some(AssemblerToken::DataStore(s@Location::Memory))) => {
                self.source.next();
                self.consume_comma()?;
                if let Some(&AssemblerToken::TwoWord(two_word)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Instruction(
                        InstructionCode::Mvi,
                        Some(InstructionArgument::DataStore(s)),
                        Some(InstructionArgument::from(two_word))
                    )))
                } else {
                    Err(Error::from(AssemblerError::ExpectingNumber {
                        got: self.source.next(),
                    }))
                }
            },
            (InstructionCode::Mvi, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Noop, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Noop, None, None))),
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
                self.parse_instruction_with_location(l, InstructionCode::Ora),
            (InstructionCode::Ora, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ori, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Ori),
            (InstructionCode::Ori, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Out, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Out),
            (InstructionCode::Out, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Pchl, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Pchl, None, None))),
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::Psw }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Pop),
            (InstructionCode::Pop, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::H }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::Psw }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Push),
            (InstructionCode::Push, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ral, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Ral, None, None))),
            (InstructionCode::Rar, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rar, None, None))),
            (InstructionCode::Rc, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rc, None, None))),
            (InstructionCode::Ret, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Ret, None, None))),
            (InstructionCode::Rlc, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rlc, None, None))),
            (InstructionCode::Rm, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rm, None, None))),
            (InstructionCode::Rnc, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rnc, None, None))),
            (InstructionCode::Rnz, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rnz, None, None))),
            (InstructionCode::Rp, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rp, None, None))),
            (InstructionCode::Rpe, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rpe, None, None))),
            (InstructionCode::Rpo, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rpo, None, None))),
            (InstructionCode::Rrc, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rrc, None, None))),
            (InstructionCode::Rst, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Rst),
            (InstructionCode::Rst, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Rz, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Rz, None, None))),
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
                self.parse_instruction_with_location(l, InstructionCode::Sbb),
            (InstructionCode::Sbb, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Sbi, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Sbi),
            (InstructionCode::Sbi, n) =>
                Err(Error::from(AssemblerError::ExpectingNumber { got: (*n).clone() })),
            (InstructionCode::Shld, _) => self.parse_two_word_instruction(InstructionCode::Shld),
            (InstructionCode::Sphl, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Sphl, None, None))),
            (InstructionCode::Sta, _) => self.parse_two_word_instruction(InstructionCode::Sta),
            (InstructionCode::Stax,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::B }))) |
            (InstructionCode::Stax,
                &Some(AssemblerToken::DataStore(l@Location::Register { register: RegisterType::D }))) =>
                self.parse_instruction_with_location(l, InstructionCode::Stax),
            (InstructionCode::Stax, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Stc, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Stc, None, None))),
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
                self.parse_instruction_with_location(l, InstructionCode::Sub),
            (InstructionCode::Sub, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Sui, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Sui),
            (InstructionCode::Sui, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Xchg, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Xchg, None, None))),
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
                self.parse_instruction_with_location(l, InstructionCode::Xra),
            (InstructionCode::Xra, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Xri, &Some(AssemblerToken::TwoWord(byte))) =>
                self.parse_instruction_with_word(byte as u8, InstructionCode::Xri),
            (InstructionCode::Xri, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Xthl, _) =>
                Ok(Statement::InstructionExprStmt(Instruction(InstructionCode::Xthl, None, None))),
        } 
    }

    #[inline]
    fn parse_instruction_with_location(&mut self, l: Location, i: InstructionCode)
        -> Result<Statement, Error> {
        self.source.next();
        Ok(Statement::InstructionExprStmt(Instruction(
            i,
            Some(InstructionArgument::DataStore(l)),
            None,
        )))
    }

    #[inline]
    fn parse_instruction_with_word(&mut self, word: u8, i: InstructionCode) -> Result<Statement, Error> {
        self.source.next();
        Ok(Statement::InstructionExprStmt(Instruction(
            i,
            Some(InstructionArgument::from(word)),
            None,
        )))
    }

    #[inline]
    fn parse_two_word_instruction(&mut self, i: InstructionCode) -> Result<Statement, Error> {
        let op = self.parse_operation()?;
        Ok(Statement::InstructionExprStmt(Instruction(
            i.clone(), Some(InstructionArgument::TwoWord(op)), None
        )))
    }
}
