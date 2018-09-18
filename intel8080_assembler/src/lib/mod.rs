#[macro_use] extern crate failure;
extern crate intel8080cpu;

use failure::{Error, Fail};
use intel8080cpu::{Intel8080Instruction, Location, RegisterType};
use std::io::{Bytes, Read};
use std::iter::Peekable;

struct Label(String);

#[derive(Debug, Fail)]
pub enum AssemblerError {
    #[fail(display = "Unexpected character: {}", c)]
    UnexpectedCharacter {
        c: char,
    },
    #[fail(display = "THERE IS SOMETHING VERY WRONG DUDE")]
    UndefinedError,
}

pub(crate) enum Intel8080AssemblerToken {
    Colon,
    Comma,
    DataStore(Location),
    Equ,
    InstructionCode(String),
    LabelToken(Label),
    Minus,
    Number(u16),
    Plus,
}

pub(crate) enum Intel8080Expression {
    Instruction(Intel8080Instruction),
    LabelDefinition(Label),
    DataDefinition { label: Label, value: u16 },
}

pub struct Lexer<R: Read> {
    source: Peekable<Bytes<R>>,
    tokens: Vec<Intel8080AssemblerToken>,
}

impl<R: Read> Lexer<R> {
    pub fn new(source: R) -> Lexer<R> {
        Lexer {
            source: source.bytes().peekable(),
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), Error> {
        while let Some(i) = self.source.next() {
            let input = i? as char;
            self.scan_token(input)?;
        }
        Ok(())
    }

    fn scan_token(&mut self, input: char) -> Result<(), Error> {
        let token = match input {
            c if c.is_whitespace() => Ok(None),
            c if c.is_digit(10) => self.maybe_scan_number(input),
            'A' | 'B' | 'C' | 'D' | 'E' | 'H' | 'L' | 'M' | 'P'  => self.maybe_scan_location(input),
            'S' => self.maybe_scan_sp_register(),
            ':' => Ok(Some(Intel8080AssemblerToken::Colon)),
            ',' => Ok(Some(Intel8080AssemblerToken::Comma)),
            '+' => Ok(Some(Intel8080AssemblerToken::Plus)),
            '-' => Ok(Some(Intel8080AssemblerToken::Minus)),
            _ => Err(Error::from(AssemblerError::UnexpectedCharacter { c: input })),
        }?;
        if let Some(t) = token {
            self.tokens.push(t);
        }
        Ok(())
    }

    #[inline]
    fn maybe_scan_number(&mut self, first_digit: char)-> Result<Option<Intel8080AssemblerToken>, Error> {
        let mut number_string = format!("{}", first_digit);
        while self.check(|c| c.is_digit(16)) {
            let digit = self.source.next().unwrap()? as char;
            number_string.push(digit);
        }
        let radix = if self.check(|c| c == 'H') { 16 } else { 10 };
        let number = u16::from_str_radix(&number_string, radix)?;
        if self.is_at_end_of_statement() {
            Ok(Some(Intel8080AssemblerToken::Number(number)))
        } else if let Some(Ok(c)) = self.source.peek() {
            Err(Error::from(AssemblerError::UnexpectedCharacter { c: (*c) as char }))
        } else {
            Err(Error::from(AssemblerError::UndefinedError))
        }
    }

    #[inline]
    fn at_end_of_statement(&mut self) -> bool {
        self.source.peek().is_none() ||
            self.check(|c| c.is_whitespace() || c == ',' || c == '+' || c == '-')
    }

    #[inline]
    fn maybe_scan_location(&mut self, input: char) -> Result<Option<Intel8080AssemblerToken>, Error> {
        if self.check(|c| c.is_whitespace() || c == ',') {
            let register_string = format!("{}", input);
            Ok(Some(Intel8080AssemblerToken::DataStore(Location::from(&register_string)?)))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn maybe_scan_sp_register(&mut self) -> Result<Option<Intel8080AssemblerToken>, Error> {
        if self.check(|c| c == 'P') {
            self.source.next().unwrap()?;
            Ok(Some(Intel8080AssemblerToken::DataStore(Location::Register {
                register: RegisterType::Sp,
            })))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn check<F: Fn(char) -> bool>(&mut self, filter: F) -> bool {
        let future = self.source.peek();
        if let Some(Ok(ref c)) = future {
            filter(*c as char)
        } else {
            false
        }
    }
}