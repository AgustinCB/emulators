extern crate failure;

use failure::Error;
use std::iter::{IntoIterator, Peekable};
use std::vec::IntoIter;
use super::{AssemblerError, AssemblerToken, Expression};

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
            (AssemblerToken::LabelToken(label), Some(AssemblerToken::Equ)) => {
                self.source.next();
                if let Some(AssemblerToken::Number(value)) = self.source.peek() {
                    Ok(Expression::DataDefinition { value: *value, label: (*label).clone() })
                } else {
                    Err(Error::from(AssemblerError::ExpectingNumber))
                }
            },
            _ => Err(Error::from(AssemblerError::UndefinedError)),
        }?;
        self.expressions.push(expression);
        Ok(())
    }
}