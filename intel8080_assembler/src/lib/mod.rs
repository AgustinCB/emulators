extern crate failure;
extern crate intel8080cpu;

use failure::Error;
use intel8080cpu::{Intel8080Instruction, Location, RegisterType};
use std::io::{Bytes, Read};
use std::iter::Peekable;

struct Label(String);

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

    fn scan_tokens(&mut self) -> Result<(), Error> {
        while let Some(i) = self.source.next() {
            let input = i? as char;
            self.scan_token(input);
        }
        Ok(())
    }

    fn scan_token(&mut self, input: char) -> Result<(), Error> {
        let token = match input {
            c if c.is_whitespace() => None,
            'A' | 'B' | 'C' | 'D' | 'E' | 'H' | 'L' | 'M' => self.maybe_scan_location(input)?,
            'S' => {
                let res = self.maybe_scan_sp_register()?;
                if let Some(_) = res {
                    self.source.next();
                }
                res
            },
            ':' => Some(Intel8080AssemblerToken::Colon),
            ',' => Some(Intel8080AssemblerToken::Comma),
            '+' => Some(Intel8080AssemblerToken::Plus),
            '-' => Some(Intel8080AssemblerToken::Minus),
            _ => panic!("Meh"),
        };
        if let Some(t) = token {
            self.tokens.push(t);
        }
        Ok(())
    }

    #[inline]
    fn maybe_scan_location(&mut self, input: char) -> Result<Option<Intel8080AssemblerToken>, Error> {
        if let Some(Ok(ref c)) = self.source.peek() {
            if (*c as char).is_whitespace() || (*c as char) == ',' {
                let register_string = format!("{}", input);
                Ok(Some(Intel8080AssemblerToken::DataStore(Location::from(&register_string)?)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn maybe_scan_sp_register(&mut self) -> Result<Option<Intel8080AssemblerToken>, Error> {
        let future = self.source.peek();
        if let Some(Ok(ref c)) = future {
            if (*c as char) == 'P' {
                Ok(Some(Intel8080AssemblerToken::DataStore(Location::Register {
                    register: RegisterType::Sp,
                })))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}