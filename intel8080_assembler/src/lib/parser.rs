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
            (InstructionCode::Adi, &Some(AssemblerToken::Byte(byte))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Adi { byte })),
            (InstructionCode::Cma, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cma)),
            (InstructionCode::Cmc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cmc)),
            (InstructionCode::Daa, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Daa)),
            (InstructionCode::Dad,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dad { register: RegisterType::B })),
            (InstructionCode::Dad,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dad { register: RegisterType::D })),
            (InstructionCode::Dad,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::H }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dad { register: RegisterType::H })),
            (InstructionCode::Dad,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::Sp }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dad { register: RegisterType::Sp })),
            (InstructionCode::Dcx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dcx { register: RegisterType::B })),
            (InstructionCode::Dcx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dcx { register: RegisterType::D })),
            (InstructionCode::Dcx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::H }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dcx { register: RegisterType::H })),
            (InstructionCode::Dcx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::Sp }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Dcx { register: RegisterType::Sp })),
            (InstructionCode::Di, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Di)),
            (InstructionCode::Ei, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ei)),
            (InstructionCode::Hlt, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Hlt)),
            (InstructionCode::Inx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Inx { register: RegisterType::B })),
            (InstructionCode::Inx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Inx { register: RegisterType::D })),
            (InstructionCode::Inx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::H }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Inx { register: RegisterType::H })),
            (InstructionCode::Inx,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::Sp }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Inx { register: RegisterType::Sp })),
            (InstructionCode::Ldax,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ldax { register: RegisterType::B })),
            (InstructionCode::Ldax,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ldax { register: RegisterType::D })),
            (InstructionCode::Noop, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Noop)),
            (InstructionCode::Pchl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pchl)),
            (InstructionCode::Pop,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pop { register: RegisterType::B })),
            (InstructionCode::Pop,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pop { register: RegisterType::D })),
            (InstructionCode::Pop,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::H }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pop { register: RegisterType::H })),
            (InstructionCode::Pop,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::Psw }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pop { register: RegisterType::Psw })),
            (InstructionCode::Push,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Push { register: RegisterType::B })),
            (InstructionCode::Push,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Push { register: RegisterType::D })),
            (InstructionCode::Push,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::H }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Push { register: RegisterType::H })),
            (InstructionCode::Push,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::Psw }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Push { register: RegisterType::Psw })),
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
            (InstructionCode::Rz, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rz)),
            (InstructionCode::Sphl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sphl)),
            (InstructionCode::Stax,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::B }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Stax { register: RegisterType::B })),
            (InstructionCode::Stax,
                Some(AssemblerToken::DataStore(Location::Register { register: RegisterType::D }))) =>
                Ok(Expression::Instruction(Intel8080Instruction::Stax { register: RegisterType::D })),
            (InstructionCode::Stc, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Stc)),
            (InstructionCode::Xchg, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xchg)),
            (InstructionCode::Xthl, _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xthl)),
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }
    }
}