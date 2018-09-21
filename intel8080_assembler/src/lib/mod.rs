#[macro_use] extern crate failure;
extern crate intel8080cpu;

use failure::Fail;
use intel8080cpu::{Intel8080Instruction, Location};

#[derive(Clone, Debug, PartialEq)]
pub struct Label(String);

#[derive(Debug, Fail)]
pub enum AssemblerError {
    #[fail(display = "Unexpected character: {}", c)]
    UnexpectedCharacter {
        c: char,
    },
    #[fail(display = "Expecting number")]
    ExpectingNumber,
    #[fail(display = "THERE IS SOMETHING VERY WRONG DUDE")]
    UndefinedError,
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum AssemblerToken {
    Byte(u8),
    Colon,
    Comma,
    DataStore(Location),
    Db,
    Dw,
    InstructionCode(InstructionCode),
    LabelToken(Label),
    Minus,
    Word(u16),
    Plus,
}

pub(crate) enum Expression {
    ByteDefinition { label: Label, value: u8 },
    Instruction(Intel8080Instruction),
    LabelDefinition(Label),
    WordDefinition { label: Label, value: u16 },
}

mod lexer;
mod parser;
pub use lexer::Lexer;
pub use parser::Parser;