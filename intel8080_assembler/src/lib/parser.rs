extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Location, Intel8080Instruction, RegisterType};
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

    pub fn parse_expressions(&mut self) -> Result<(), Error> {
        while let Some(input) = self.source.next() {
            self.parse_expression(&input)?;
        }
        Ok(())
    }

    fn parse_expression(&mut self, input: &AssemblerToken) -> Result<(), Error> {
        let next = self.source.peek().map(|a| (*a).clone());
        let expression = match (input, next) {
            (AssemblerToken::Org, Some(AssemblerToken::TwoWord(value))) => {
                self.source.next();
                self.parse_statement_with_two_words_operation(
                    TwoWordExpression::Literal(value),
                    |o| Statement::OrgStatement(o),
                    |v| Statement::OrgStatement(TwoWordValue::Operand(v)))
            },
            (AssemblerToken::Org, Some(AssemblerToken::LabelToken(label))) => {
                self.source.next();
                self.parse_statement_with_two_words_operation(
                    TwoWordExpression::Label(label),
                    |o| Statement::OrgStatement(o),
                    |v| Statement::OrgStatement(TwoWordValue::Operand(v)))
            },
            (&AssemblerToken::LabelToken(ref label), Some(AssemblerToken::Colon)) => {
                self.source.next();
                Ok(Statement::LabelDefinitionStatement(label.clone()))
            },
            (&AssemblerToken::LabelToken(ref label), Some(AssemblerToken::Dw)) =>
                self.parse_word_definition(label),
            (&AssemblerToken::LabelToken(ref label), Some(AssemblerToken::Db)) =>
                self.parse_byte_definition(label),
            (AssemblerToken::InstructionCode(instruction), ref next) =>
                self.parse_instruction(instruction, next),
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }?;
        self.expressions.push(expression);
        Ok(())
    }

    fn parse_byte_definition(&mut self, label: &LabelExpression) -> Result<Statement, Error> {
        self.source.next();
        let next = self.source.peek().map(|t| (*t).clone());
        let res = match next {
            Some(AssemblerToken::Word(value)) =>
                self.parse_statement_with_word_operation(
                    WordExpression::Literal(value),
                    |o| Statement::WordDefinitionStatement(label.clone(), o),
                    |v| Statement::WordDefinitionStatement(label.clone(), WordValue::Operand(v))
                ),
            Some(AssemblerToken::LabelToken(ref value_label)) =>
                self.parse_statement_with_word_operation(
                    WordExpression::Label(value_label.clone()),
                    |o| Statement::WordDefinitionStatement(label.clone(), o),
                    |v| Statement::WordDefinitionStatement(label.clone(), WordValue::Operand(v))
                ),
            _ => Err(Error::from(AssemblerError::ExpectingNumber)),
        }?;
        self.source.next();
        Ok(res)
    }

    fn parse_word_definition(&mut self, label: &LabelExpression) -> Result<Statement, Error> {
        self.source.next();
        let next = self.source.peek().map(|t| (*t).clone());
        let res = match next {
            Some(AssemblerToken::TwoWord(value)) =>
                self.parse_statement_with_two_words_operation(
                    TwoWordExpression::Literal(value),
                    |o| Statement::TwoWordDefinitionStatement(label.clone(), o),
                    |v| Statement::TwoWordDefinitionStatement(label.clone(), TwoWordValue::Operand(v))
                ),
            Some(AssemblerToken::LabelToken(ref value_label)) =>
                self.parse_statement_with_two_words_operation(
                    TwoWordExpression::Label(value_label.clone()),
                    |o| Statement::TwoWordDefinitionStatement(label.clone(), o),
                    |v| Statement::TwoWordDefinitionStatement(label.clone(), TwoWordValue::Operand(v))
                ),
            _ => Err(Error::from(AssemblerError::ExpectingNumber)),
        }?;
        self.source.next();
        Ok(res)
    }

    fn parse_statement_with_word_operation<O, D>
    (&mut self, value: WordExpression, op: O, default: D) -> Result<Statement, Error>
        where O: FnOnce(WordValue) -> Statement,
              D: FnOnce(WordExpression) -> Statement {
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(ref op_token@AssemblerToken::Plus) |
            Some(ref op_token@AssemblerToken::Minus) => {
                self.source.next();
                let r = self.parse_word_operands(op_token, value)?;
                self.source.next();
                Ok(op(r))
            },
            _ => Ok(default(value)),
        }
    }

    fn parse_statement_with_two_words_operation<O, D>
        (&mut self, value: TwoWordExpression, op: O, default: D) -> Result<Statement, Error>
        where O: FnOnce(TwoWordValue) -> Statement,
              D: FnOnce(TwoWordExpression) -> Statement {
        let next = self.source.peek().map(|t| (*t).clone());
        match next {
            Some(ref op_token@AssemblerToken::Plus) |
            Some(ref op_token@AssemblerToken::Minus) => {
                self.source.next();
                let r = self.parse_two_words_operands(op_token, value)?;
                self.source.next();
                Ok(op(r))
            },
            _ => Ok(default(value)),
        }
    }

    fn parse_word_operands(&mut self, operation: &AssemblerToken, value: WordExpression)
                                -> Result<WordValue, Error> {
        macro_rules! operation {
            ($op:ident,$value:ident,$operand:expr) => {
                if let AssemblerToken::Plus = $op {
                    Ok(WordValue::Sum($value, $operand))
                } else if let AssemblerToken::Minus = $op {
                    Ok(WordValue::Rest($value, $operand))
                } else {
                    Err(Error::from(AssemblerError::InvalidOperationToken))
                }
            }
        }
        match self.source.peek() {
            Some(&AssemblerToken::Word(other_value)) => {
                operation!(
                    operation,
                    value,
                    WordExpression::Literal(other_value)
                )
            },
            Some(&AssemblerToken::LabelToken(ref other_label)) => {
                operation!(
                    operation,
                    value,
                    WordExpression::Label(other_label.clone())
                )
            },
            _ => Err(Error::from(AssemblerError::ExpectingNumber)),
        }
    }

    fn parse_two_words_operands(&mut self, operation: &AssemblerToken, value: TwoWordExpression)
                                -> Result<TwoWordValue, Error> {
        macro_rules! operation {
            ($op:ident,$value:ident,$operand:expr) => {
                if let AssemblerToken::Plus = $op {
                    Ok(TwoWordValue::Sum($value, $operand))
                } else if let AssemblerToken::Minus = $op {
                    Ok(TwoWordValue::Rest($value, $operand))
                } else {
                    Err(Error::from(AssemblerError::InvalidOperationToken))
                }
            }
        }
        match self.source.peek() {
            Some(&AssemblerToken::TwoWord(other_value)) => {
                operation!(
                    operation,
                    value,
                    TwoWordExpression::Literal(other_value)
                )
            },
            Some(&AssemblerToken::LabelToken(ref other_label)) => {
                operation!(
                    operation,
                    value,
                    TwoWordExpression::Label(other_label.clone())
                )
            },
            _ => Err(Error::from(AssemblerError::ExpectingNumber)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Adc { source: l }))
            },
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Add { source: l }))
            },
            (InstructionCode::Add, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Aci, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Aci { byte }))
            },
            (InstructionCode::Aci, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Adi, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Adi { byte }))
            },
            (InstructionCode::Adi, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ana { source: l }))
            },
            (InstructionCode::Ana, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ani, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ani { byte }))
            },
            (InstructionCode::Ani, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Call, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Call {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Call, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cc, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cc, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cm, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cm {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cm, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cma, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cma)),
            (InstructionCode::Cmc, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cmc)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cmp { source: l }))
            },
            (InstructionCode::Cmp, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cpi, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cpi { byte }))
            },
            (InstructionCode::Cpi, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cnc, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cnc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cnc, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cnz, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cnz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cnz, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cp, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cp {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cp, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cpe, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cpe {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cpe, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cpo, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cpo {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cpo, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Cz, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Cz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Cz, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Daa, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Daa)),
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Dad,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Dad { register: r }))
            },
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Dcr { source: l }))
            },
            (InstructionCode::Dcr, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Dcx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Dcx { register: r }))
            },
            (InstructionCode::Dcx, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Di, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Di)),
            (InstructionCode::Ei, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ei)),
            (InstructionCode::Hlt, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Hlt)),
            (InstructionCode::In, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::In { byte }))
            },
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Inr { source: l }))
            },
            (InstructionCode::Inr, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Inx,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Inx { register: r }))
            },
            (InstructionCode::Inx, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jc, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jc, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jm, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jm {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jm, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jmp, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jmp {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jmp, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jnc, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jnc {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jnc, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jnz, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jnz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jnz, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jp, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jp {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jp, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jpe, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jpe {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jpe, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jpo, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jpo {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jpo, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Jz, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Jz {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Jz, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Lda, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Lda {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Lda, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ldax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Ldax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ldax { register: r }))
            },
            (InstructionCode::Ldax, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Lhld, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Lhld {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Lhld, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Lxi,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Sp}))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(low_byte)) = self.source.peek() {
                    self.source.next();
                    if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                        self.source.next();
                        Ok(Statement::InstructionExprStmt(Intel8080Instruction::Lxi {
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
            (InstructionCode::Lxi, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
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
                        Ok(Statement::InstructionExprStmt(Intel8080Instruction::Mov {
                            destiny: d,
                            source: s,
                        }))
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
                if let Some(&AssemblerToken::Word(byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Mvi {
                        byte,
                        source: s,
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Mvi, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Noop, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Noop)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ora { source: l }))
            },
            (InstructionCode::Ora, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ori, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ori { byte }))
            },
            (InstructionCode::Ori, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Out, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Out { byte }))
            },
            (InstructionCode::Out, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Pchl, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Pchl)),
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Pop,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Psw }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Pop { register: r }))
            },
            (InstructionCode::Pop, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::H }))) |
            (InstructionCode::Push,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::Psw }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Push { register: r }))
            },
            (InstructionCode::Push, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Ral, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ral)),
            (InstructionCode::Rar, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rar)),
            (InstructionCode::Rc, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rc)),
            (InstructionCode::Ret, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Ret)),
            (InstructionCode::Rlc, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rlc)),
            (InstructionCode::Rm, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rm)),
            (InstructionCode::Rnc, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rnc)),
            (InstructionCode::Rnz, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rnz)),
            (InstructionCode::Rp, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rp)),
            (InstructionCode::Rpe, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rpe)),
            (InstructionCode::Rpo, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rpo)),
            (InstructionCode::Rrc, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rrc)),
            (InstructionCode::Rst, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rst { byte }))
            },
            (InstructionCode::Rst, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Rz, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Rz)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Sbb { source: l }))
            },
            (InstructionCode::Sbb, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Sbi, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Sbi { byte }))
            },
            (InstructionCode::Sbi, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Shld, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Shld {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Shld, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Sphl, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Sphl)),
            (InstructionCode::Sta, &Some(AssemblerToken::Word(low_byte))) => {
                self.source.next();
                if let Some(&AssemblerToken::Word(high_byte)) = self.source.peek() {
                    self.source.next();
                    Ok(Statement::InstructionExprStmt(Intel8080Instruction::Sta {
                        address: [ low_byte, high_byte ],
                    }))
                } else {
                    Err(Error::from(AssemblerError::InvalidInstructionArgument))
                }
            },
            (InstructionCode::Sta, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Stax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::B }))) |
            (InstructionCode::Stax,
                &Some(AssemblerToken::DataStore(Location::Register { register: r@RegisterType::D }))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Stax { register: r }))
            },
            (InstructionCode::Stax, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Stc, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Stc)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Sub { source: l }))
            },
            (InstructionCode::Sub, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Sui, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Sui { byte }))
            },
            (InstructionCode::Sui, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Xchg, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Xchg)),
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
                &Some(AssemblerToken::DataStore(l@Location::Memory))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Xra { source: l }))
            },
            (InstructionCode::Xra, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Xri, &Some(AssemblerToken::Word(byte))) => {
                self.source.next();
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Xri { byte }))
            },
            (InstructionCode::Xri, _) =>
                Err(Error::from(AssemblerError::InvalidInstructionArgument)),
            (InstructionCode::Xthl, _) =>
                Ok(Statement::InstructionExprStmt(Intel8080Instruction::Xthl)),
        }
    }
}