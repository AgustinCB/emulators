extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::Intel8080Instruction;
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
        let expression = match (input, self.source.peek()) {
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
            (AssemblerToken::InstructionCode(InstructionCode::Cma), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cma)),
            (AssemblerToken::InstructionCode(InstructionCode::Cmc), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Cmc)),
            (AssemblerToken::InstructionCode(InstructionCode::Daa), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Daa)),
            (AssemblerToken::InstructionCode(InstructionCode::Di), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Di)),
            (AssemblerToken::InstructionCode(InstructionCode::Ei), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ei)),
            (AssemblerToken::InstructionCode(InstructionCode::Hlt), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Hlt)),
            (AssemblerToken::InstructionCode(InstructionCode::Noop), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Noop)),
            (AssemblerToken::InstructionCode(InstructionCode::Pchl), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Pchl)),
            (AssemblerToken::InstructionCode(InstructionCode::Ral), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ral)),
            (AssemblerToken::InstructionCode(InstructionCode::Rar), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rar)),
            (AssemblerToken::InstructionCode(InstructionCode::Rc), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rc)),
            (AssemblerToken::InstructionCode(InstructionCode::Ret), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Ret)),
            (AssemblerToken::InstructionCode(InstructionCode::Rlc), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rlc)),
            (AssemblerToken::InstructionCode(InstructionCode::Rm), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rm)),
            (AssemblerToken::InstructionCode(InstructionCode::Rnc), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rnc)),
            (AssemblerToken::InstructionCode(InstructionCode::Rnz), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rnz)),
            (AssemblerToken::InstructionCode(InstructionCode::Rp), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rp)),
            (AssemblerToken::InstructionCode(InstructionCode::Rpe), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rpe)),
            (AssemblerToken::InstructionCode(InstructionCode::Rpo), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rpo)),
            (AssemblerToken::InstructionCode(InstructionCode::Rrc), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rrc)),
            (AssemblerToken::InstructionCode(InstructionCode::Rz), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Rz)),
            (AssemblerToken::InstructionCode(InstructionCode::Sphl), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Sphl)),
            (AssemblerToken::InstructionCode(InstructionCode::Stc), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Stc)),
            (AssemblerToken::InstructionCode(InstructionCode::Xchg), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xchg)),
            (AssemblerToken::InstructionCode(InstructionCode::Xthl), _) =>
                Ok(Expression::Instruction(Intel8080Instruction::Xthl)),
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }?;
        self.expressions.push(expression);
        Ok(())
    }
}