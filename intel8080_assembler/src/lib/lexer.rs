use intel8080cpu::Location;
use std::io::{Bytes, Read};
use std::iter::Peekable;
use super::{InstructionCode, AssemblerToken, Label, AssemblerError};
use super::failure::Error;

pub struct Lexer<R: Read> {
    source: Peekable<Bytes<R>>,
    tokens: Vec<AssemblerToken>,
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
            ':' => Ok(Some(AssemblerToken::Colon)),
            ',' => Ok(Some(AssemblerToken::Comma)),
            '+' => Ok(Some(AssemblerToken::Plus)),
            '-' => Ok(Some(AssemblerToken::Minus)),
            _ => Err(Error::from(AssemblerError::UnexpectedCharacter { c: input })),
        }?;
        if let Some(t) = token {
            self.tokens.push(t);
        }
        Ok(())
    }

    #[inline]
    fn either_label_or_keyword(&mut self, first_char: char)
                               -> Result<Option<AssemblerToken>, Error> {
        let rest = self.consume(|c| c.is_alphabetic() || c == '_')?;
        let literal = format!("{}{}", first_char, rest);
        Ok(match literal.as_str() {
            "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "PSW" | "SP" =>
                Some(AssemblerToken::DataStore(Location::from(&literal)?)),
            "DW" => Some(AssemblerToken::Dw),
            "NOP" => Some(AssemblerToken::InstructionCode(InstructionCode::Noop)),
            "LXI" => Some(AssemblerToken::InstructionCode(InstructionCode::Lxi)),
            "STAX" => Some(AssemblerToken::InstructionCode(InstructionCode::Stax)),
            "INX" => Some(AssemblerToken::InstructionCode(InstructionCode::Inx)),
            "INR" => Some(AssemblerToken::InstructionCode(InstructionCode::Inr)),
            "DCR" => Some(AssemblerToken::InstructionCode(InstructionCode::Dcr)),
            "MVI" => Some(AssemblerToken::InstructionCode(InstructionCode::Mvi)),
            "RLC" => Some(AssemblerToken::InstructionCode(InstructionCode::Rlc)),
            "DAD" => Some(AssemblerToken::InstructionCode(InstructionCode::Dad)),
            "LDAX" => Some(AssemblerToken::InstructionCode(InstructionCode::Ldax)),
            "DCX" => Some(AssemblerToken::InstructionCode(InstructionCode::Dcx)),
            "RRC" => Some(AssemblerToken::InstructionCode(InstructionCode::Rrc)),
            "RAL" => Some(AssemblerToken::InstructionCode(InstructionCode::Ral)),
            "RAR" => Some(AssemblerToken::InstructionCode(InstructionCode::Rar)),
            "SHLD" => Some(AssemblerToken::InstructionCode(InstructionCode::Shld)),
            "DAA" => Some(AssemblerToken::InstructionCode(InstructionCode::Daa)),
            "LHLD" => Some(AssemblerToken::InstructionCode(InstructionCode::Lhld)),
            "CMA" => Some(AssemblerToken::InstructionCode(InstructionCode::Cma)),
            "STA" => Some(AssemblerToken::InstructionCode(InstructionCode::Sta)),
            "LDA" => Some(AssemblerToken::InstructionCode(InstructionCode::Lda)),
            "STC" => Some(AssemblerToken::InstructionCode(InstructionCode::Stc)),
            "CMC" => Some(AssemblerToken::InstructionCode(InstructionCode::Cmc)),
            "MOV" => Some(AssemblerToken::InstructionCode(InstructionCode::Mov)),
            "HLT" => Some(AssemblerToken::InstructionCode(InstructionCode::Hlt)),
            "ADD" => Some(AssemblerToken::InstructionCode(InstructionCode::Add)),
            "ADC" => Some(AssemblerToken::InstructionCode(InstructionCode::Adc)),
            "SUB" => Some(AssemblerToken::InstructionCode(InstructionCode::Sub)),
            "SBB" => Some(AssemblerToken::InstructionCode(InstructionCode::Sbb)),
            "ANA" => Some(AssemblerToken::InstructionCode(InstructionCode::Ana)),
            "XRA" => Some(AssemblerToken::InstructionCode(InstructionCode::Xra)),
            "ORA" => Some(AssemblerToken::InstructionCode(InstructionCode::Ora)),
            "CMP" => Some(AssemblerToken::InstructionCode(InstructionCode::Cmp)),
            "RNZ" => Some(AssemblerToken::InstructionCode(InstructionCode::Rnz)),
            "POP" => Some(AssemblerToken::InstructionCode(InstructionCode::Pop)),
            "JNZ" => Some(AssemblerToken::InstructionCode(InstructionCode::Jnz)),
            "JMP" => Some(AssemblerToken::InstructionCode(InstructionCode::Jmp)),
            "CNZ" => Some(AssemblerToken::InstructionCode(InstructionCode::Cnz)),
            "PUSH" => Some(AssemblerToken::InstructionCode(InstructionCode::Push)),
            "ADI" => Some(AssemblerToken::InstructionCode(InstructionCode::Adi)),
            "RST" => Some(AssemblerToken::InstructionCode(InstructionCode::Rst)),
            "RZ" => Some(AssemblerToken::InstructionCode(InstructionCode::Rz)),
            "RET" => Some(AssemblerToken::InstructionCode(InstructionCode::Ret)),
            "JZ" => Some(AssemblerToken::InstructionCode(InstructionCode::Jz)),
            "CZ" => Some(AssemblerToken::InstructionCode(InstructionCode::Cz)),
            "CALL" => Some(AssemblerToken::InstructionCode(InstructionCode::Call)),
            "ACI" => Some(AssemblerToken::InstructionCode(InstructionCode::Aci)),
            "RNC" => Some(AssemblerToken::InstructionCode(InstructionCode::Rnc)),
            "JNC" => Some(AssemblerToken::InstructionCode(InstructionCode::Jnc)),
            "OUT" => Some(AssemblerToken::InstructionCode(InstructionCode::Out)),
            "CNC" => Some(AssemblerToken::InstructionCode(InstructionCode::Cnc)),
            "SUI" => Some(AssemblerToken::InstructionCode(InstructionCode::Sui)),
            "RC" => Some(AssemblerToken::InstructionCode(InstructionCode::Rc)),
            "JC" => Some(AssemblerToken::InstructionCode(InstructionCode::Jc)),
            "IN" => Some(AssemblerToken::InstructionCode(InstructionCode::In)),
            "CC" => Some(AssemblerToken::InstructionCode(InstructionCode::Cc)),
            "SBI" => Some(AssemblerToken::InstructionCode(InstructionCode::Sbi)),
            "RPO" => Some(AssemblerToken::InstructionCode(InstructionCode::Rpo)),
            "JPO" => Some(AssemblerToken::InstructionCode(InstructionCode::Jpo)),
            "XTHL" => Some(AssemblerToken::InstructionCode(InstructionCode::Xthl)),
            "CPO" => Some(AssemblerToken::InstructionCode(InstructionCode::Cpo)),
            "ANI" => Some(AssemblerToken::InstructionCode(InstructionCode::Ani)),
            "RPE" => Some(AssemblerToken::InstructionCode(InstructionCode::Rpe)),
            "PCHL" => Some(AssemblerToken::InstructionCode(InstructionCode::Pchl)),
            "JPE" => Some(AssemblerToken::InstructionCode(InstructionCode::Jpe)),
            "XCHG" => Some(AssemblerToken::InstructionCode(InstructionCode::Xchg)),
            "CPE" => Some(AssemblerToken::InstructionCode(InstructionCode::Cpe)),
            "XRI" => Some(AssemblerToken::InstructionCode(InstructionCode::Xri)),
            "RP" => Some(AssemblerToken::InstructionCode(InstructionCode::Rp)),
            "JP" => Some(AssemblerToken::InstructionCode(InstructionCode::Jp)),
            "DI" => Some(AssemblerToken::InstructionCode(InstructionCode::Di)),
            "CP" => Some(AssemblerToken::InstructionCode(InstructionCode::Cp)),
            "ORI" => Some(AssemblerToken::InstructionCode(InstructionCode::Ori)),
            "RM" => Some(AssemblerToken::InstructionCode(InstructionCode::Rm)),
            "SPHL" => Some(AssemblerToken::InstructionCode(InstructionCode::Sphl)),
            "JM" => Some(AssemblerToken::InstructionCode(InstructionCode::Jm)),
            "EI" => Some(AssemblerToken::InstructionCode(InstructionCode::Ei)),
            "CM" => Some(AssemblerToken::InstructionCode(InstructionCode::Cm)),
            "CPI" => Some(AssemblerToken::InstructionCode(InstructionCode::Cpi)),
            _ => Some(AssemblerToken::LabelToken(Label(literal)))
        })
    }

    #[inline]
    fn maybe_scan_number(&mut self, first_digit: char)-> Result<Option<AssemblerToken>, Error> {
        let rest = self.consume(|c| c.is_digit(16))?;
        let number_string = format!("{}{}", first_digit, rest);
        let radix = if self.check(|c| c == 'H') { 16 } else { 10 };
        let number = u16::from_str_radix(&number_string, radix)?;
        if self.at_end_of_statement() {
            Ok(Some(AssemblerToken::Number(number)))
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