use super::failure::Error;
use super::{AssemblerError, AssemblerToken, AssemblerTokenType, InstructionCode, LabelExpression};
use intel8080cpu::Location;
use std::io::{Bytes, Read};
use std::iter::Peekable;
use std::str::FromStr;

pub struct Lexer<R: Read> {
    source: Peekable<Bytes<R>>,
    tokens: Vec<AssemblerToken>,
    line: usize,
}

impl<R: Read> Lexer<R> {
    pub fn new(source: R) -> Lexer<R> {
        Lexer {
            source: source.bytes().peekable(),
            tokens: Vec::new(),
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<AssemblerToken>, Error> {
        while let Some(i) = self.source.next() {
            let input = i? as char;
            self.scan_token(input)?;
        }
        Ok(self.tokens)
    }

    fn scan_token(&mut self, input: char) -> Result<(), Error> {
        let token: Option<AssemblerTokenType> = match input {
            '\n' => {
                self.line += 1;
                Ok(None)
            }
            c if c.is_whitespace() => Ok(None),
            c if c.is_digit(10) => self.maybe_scan_number(input),
            c if c.is_alphabetic() || c == '?' || c == '@' => self.either_label_or_keyword(input),
            '\'' => self.scan_char(),
            '(' => Ok(Some(AssemblerTokenType::LeftParen)),
            ')' => Ok(Some(AssemblerTokenType::RightParen)),
            ':' => Ok(Some(AssemblerTokenType::Colon)),
            ';' => {
                self.consume(|c| c != '\n')?;
                Ok(None)
            }
            ',' => Ok(Some(AssemblerTokenType::Comma)),
            '+' => Ok(Some(AssemblerTokenType::Plus)),
            '-' => Ok(Some(AssemblerTokenType::Minus)),
            '$' => Ok(Some(AssemblerTokenType::Dollar)),
            '*' => Ok(Some(AssemblerTokenType::Mult)),
            '/' => Ok(Some(AssemblerTokenType::Div)),
            _ => Err(Error::from(AssemblerError::UnexpectedCharacter {
                c: input,
                line: self.line,
            })),
        }?;
        if let Some(t) = token {
            self.tokens.push(AssemblerToken {
                token_type: t,
                line: self.line,
            });
        }
        Ok(())
    }

    #[inline]
    fn scan_char(&mut self) -> Result<Option<AssemblerTokenType>, Error> {
        let rest = self.consume(|c| c != '\'')?;
        self.source.next();
        let value = char::from_str(&rest)?;
        Ok(Some(AssemblerTokenType::Char(value)))
    }

    #[inline]
    fn either_label_or_keyword(
        &mut self,
        first_char: char,
    ) -> Result<Option<AssemblerTokenType>, Error> {
        let rest = self.consume(|c| c.is_alphabetic() || c == '_')?;
        let literal = format!("{}{}", first_char, rest);
        Ok(match literal.as_str() {
            "A" | "B" | "C" | "D" | "E" | "H" | "L" | "M" | "PSW" | "SP" => {
                Some(AssemblerTokenType::DataStore(Location::from(&literal)?))
            }
            "AND" => Some(AssemblerTokenType::And),
            "DB" => Some(AssemblerTokenType::Db),
            "DW" => Some(AssemblerTokenType::Dw),
            "ORG" => Some(AssemblerTokenType::Org),
            "MOD" => Some(AssemblerTokenType::Mod),
            "NOT" => Some(AssemblerTokenType::Not),
            "OR" => Some(AssemblerTokenType::Or),
            "SHL" => Some(AssemblerTokenType::Shl),
            "SHR" => Some(AssemblerTokenType::Shr),
            "XOR" => Some(AssemblerTokenType::Xor),
            "NOP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Noop)),
            "LXI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Lxi)),
            "STAX" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Stax)),
            "INX" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Inx)),
            "INR" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Inr)),
            "DCR" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Dcr)),
            "MVI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Mvi)),
            "RLC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rlc)),
            "DAD" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Dad)),
            "LDAX" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ldax)),
            "DCX" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Dcx)),
            "RRC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rrc)),
            "RAL" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ral)),
            "RAR" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rar)),
            "SHLD" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Shld)),
            "DAA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Daa)),
            "LHLD" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Lhld)),
            "CMA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cma)),
            "STA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Sta)),
            "LDA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Lda)),
            "STC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Stc)),
            "CMC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cmc)),
            "MOV" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Mov)),
            "HLT" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Hlt)),
            "ADD" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Add)),
            "ADC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Adc)),
            "SUB" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Sub)),
            "SBB" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Sbb)),
            "ANA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ana)),
            "XRA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Xra)),
            "ORA" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ora)),
            "CMP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cmp)),
            "RNZ" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rnz)),
            "POP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Pop)),
            "JNZ" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jnz)),
            "JMP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jmp)),
            "CNZ" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cnz)),
            "PUSH" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Push)),
            "ADI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Adi)),
            "RST" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rst)),
            "RZ" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rz)),
            "RET" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ret)),
            "JZ" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jz)),
            "CZ" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cz)),
            "CALL" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Call)),
            "ACI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Aci)),
            "RNC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rnc)),
            "JNC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jnc)),
            "OUT" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Out)),
            "CNC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cnc)),
            "SUI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Sui)),
            "RC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rc)),
            "JC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jc)),
            "IN" => Some(AssemblerTokenType::InstructionCode(InstructionCode::In)),
            "CC" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cc)),
            "SBI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Sbi)),
            "RPO" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rpo)),
            "JPO" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jpo)),
            "XTHL" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Xthl)),
            "CPO" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cpo)),
            "ANI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ani)),
            "RPE" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rpe)),
            "PCHL" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Pchl)),
            "JPE" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jpe)),
            "XCHG" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Xchg)),
            "CPE" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cpe)),
            "XRI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Xri)),
            "RP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rp)),
            "JP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jp)),
            "DI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Di)),
            "CP" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cp)),
            "ORI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ori)),
            "RM" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Rm)),
            "SPHL" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Sphl)),
            "JM" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Jm)),
            "EI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Ei)),
            "CM" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cm)),
            "CPI" => Some(AssemblerTokenType::InstructionCode(InstructionCode::Cpi)),
            _ => Some(AssemblerTokenType::LabelToken(LabelExpression(literal))),
        })
    }

    #[inline]
    fn maybe_scan_number(
        &mut self,
        first_digit: char,
    ) -> Result<Option<AssemblerTokenType>, Error> {
        let rest = self.consume(char::is_alphanumeric)?;
        let mut number_string = format!("{}{}", first_digit, rest);
        let radix_marker = number_string.pop().unwrap(); // Safe because len(number_string) > 0
        let radix = if radix_marker == 'H' {
            16
        } else if radix_marker == 'O' || radix_marker == 'Q' {
            8
        } else if radix_marker == 'B' {
            2
        } else {
            number_string.push(radix_marker);
            10
        };
        let number = u16::from_str_radix(&number_string, radix)?;
        Ok(Some(AssemblerTokenType::TwoWord(number)))
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
