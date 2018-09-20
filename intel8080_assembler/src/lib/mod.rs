#[macro_use] extern crate failure;
extern crate intel8080cpu;

use failure::{Error, Fail};
use intel8080cpu::{Intel8080Instruction, Location};
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

pub enum InstructionCode {
    Noop,
    Lxi,
    Stax,
    Inx,
    Inr,
    Dcr,
    Mvi,
    Rlc,
    Dad,
    Ldax,
    Dcx,
    Rrc,
    Ral,
    Rar,
    Shld,
    Daa,
    Lhld,
    Cma,
    Sta,
    Lda,
    Stc,
    Cmc,
    Mov,
    Hlt,
    Add,
    Adc,
    Sub,
    Sbb,
    Ana,
    Xra,
    Ora,
    Cmp,
    Rnz,
    Pop,
    Jnz,
    Jmp,
    Cnz,
    Push,
    Adi,
    Rst,
    Rz,
    Ret,
    Jz,
    Cz,
    Call,
    Aci,
    Rnc,
    Jnc,
    Out,
    Cnc,
    Sui,
    Rc,
    Jc,
    In,
    Cc,
    Sbi,
    Rpo,
    Jpo,
    Xthl,
    Cpo,
    Ani,
    Rpe,
    Pchl,
    Jpe,
    Xchg,
    Cpe,
    Xri,
    Rp,
    Jp,
    Di,
    Cp,
    Ori,
    Rm,
    Sphl,
    Jm,
    Ei,
    Cm,
    Cpi,
}

pub(crate) enum Intel8080AssemblerToken {
    Colon,
    Comma,
    DataStore(Location),
    Equ,
    InstructionCode(InstructionCode),
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
            c if c.is_alphabetic() || c == '_' => self.either_label_or_keyword(input),
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
    fn either_label_or_keyword(&mut self, first_char: char)
        -> Result<Option<Intel8080AssemblerToken>, Error> {
        let rest = self.consume(|c| c.is_alphabetic() || c == '_')?;
        let literal = format!("{}{}", first_char, rest);
        Ok(match literal.as_str() {
            "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "PSW" | "SP" =>
                Some(Intel8080AssemblerToken::DataStore(Location::from(&literal)?)),
            "EQU" => Some(Intel8080AssemblerToken::Equ),
            "NOP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Noop)),
            "LXI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Lxi)),
            "STAX" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Stax)),
            "INX" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Inx)),
            "INR" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Inr)),
            "DCR" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Dcr)),
            "MVI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Mvi)),
            "RLC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rlc)),
            "DAD" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Dad)),
            "LDAX" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ldax)),
            "DCX" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Dcx)),
            "RRC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rrc)),
            "RAL" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ral)),
            "RAR" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rar)),
            "SHLD" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Shld)),
            "DAA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Daa)),
            "LHLD" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Lhld)),
            "CMA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cma)),
            "STA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Sta)),
            "LDA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Lda)),
            "STC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Stc)),
            "CMC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cmc)),
            "MOV" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Mov)),
            "HLT" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Hlt)),
            "ADD" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Add)),
            "ADC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Adc)),
            "SUB" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Sub)),
            "SBB" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Sbb)),
            "ANA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ana)),
            "XRA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Xra)),
            "ORA" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ora)),
            "CMP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cmp)),
            "RNZ" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rnz)),
            "POP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Pop)),
            "JNZ" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jnz)),
            "JMP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jmp)),
            "CNZ" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cnz)),
            "PUSH" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Push)),
            "ADI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Adi)),
            "RST" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rst)),
            "RZ" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rz)),
            "RET" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ret)),
            "JZ" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jz)),
            "CZ" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cz)),
            "CALL" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Call)),
            "ACI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Aci)),
            "RNC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rnc)),
            "JNC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jnc)),
            "OUT" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Out)),
            "CNC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cnc)),
            "SUI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Sui)),
            "RC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rc)),
            "JC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jc)),
            "IN" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::In)),
            "CC" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cc)),
            "SBI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Sbi)),
            "RPO" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rpo)),
            "JPO" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jpo)),
            "XTHL" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Xthl)),
            "CPO" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cpo)),
            "ANI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ani)),
            "RPE" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rpe)),
            "PCHL" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Pchl)),
            "JPE" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jpe)),
            "XCHG" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Xchg)),
            "CPE" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cpe)),
            "XRI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Xri)),
            "RP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rp)),
            "JP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jp)),
            "DI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Di)),
            "CP" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cp)),
            "ORI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ori)),
            "RM" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Rm)),
            "SPHL" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Sphl)),
            "JM" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Jm)),
            "EI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Ei)),
            "CM" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cm)),
            "CPI" => Some(Intel8080AssemblerToken::InstructionCode(InstructionCode::Cpi)),
            _ => Some(Intel8080AssemblerToken::LabelToken(Label(literal)))
        })
    }

    #[inline]
    fn maybe_scan_number(&mut self, first_digit: char)-> Result<Option<Intel8080AssemblerToken>, Error> {
        let rest = self.consume(|c| c.is_digit(16))?;
        let number_string = format!("{}{}", first_digit, rest);
        let radix = if self.check(|c| c == 'H') { 16 } else { 10 };
        let number = u16::from_str_radix(&number_string, radix)?;
        if self.at_end_of_statement() {
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
    fn consume<F: Fn(char) -> bool + Copy>(&mut self, while_condition: F) -> Result<String, Error> {
        let mut result = String::from("");
        while self.check(while_condition) {
            let next = self.source.next().unwrap()? as char;
            result.push(next);
        }
        Ok(result)
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