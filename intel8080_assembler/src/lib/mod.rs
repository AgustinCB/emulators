extern crate intel8080cpu;

use intel8080cpu::{Location, Intel8080Instruction};
use std::io::{Read, Result};

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

pub struct Lexer<T: Read> {
    source: T,
    tokens: Vec<Intel8080AssemblerToken>,
}

impl<T: Read> Lexer<T> {
    pub fn new(source: T) -> Lexer<T> {
        Lexer {
            source,
            tokens: Vec::new(),
        }
    }

    fn scan_token(&mut self) -> Result<()> {
        let mut buffer: [u8; 1] = [0; 1];
        self.source.read_exact(&mut buffer)?;
        let token = match buffer[0] as char {
            ':' => Intel8080AssemblerToken::Colon,
            ',' => Intel8080AssemblerToken::Comma,
            '+' => Intel8080AssemblerToken::Plus,
            '-' => Intel8080AssemblerToken::Minus,
            _ => panic!("Meh"),
        };
        self.tokens.push(token);
        Ok(())
    }
}