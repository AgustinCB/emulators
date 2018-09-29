#[macro_use] extern crate failure;
extern crate intel8080cpu;

use intel8080cpu::{Intel8080Instruction, Location};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Label(String);

#[derive(Debug, Fail)]
pub enum AssemblerError {
    #[fail(display = "Unexpected character: {}", c)]
    UnexpectedCharacter {
        c: char,
    },
    #[fail(display = "Expecting number")]
    ExpectingNumber,
    #[fail(display = "Invalid argument for instruction")]
    InvalidInstructionArgument,
    #[fail(display = "THERE IS SOMETHING VERY WRONG DUDE")]
    UndefinedError,
    #[fail(display = "Label doesn't exist.")]
    LabelDoesntExist,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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
    Org,
    Word(u16),
    Plus,
}

pub enum ByteOperand {
    Literal(u8),
    Label(Label),
}

pub enum ByteValue {
    Operand(ByteOperand),
}

pub enum WordOperand {
    Literal(u16),
    Label(Label),
}

pub enum WordValue {
    Operand(WordOperand),
    Sum(WordOperand, WordOperand)
}

pub enum Expression {
    ByteDefinition { label: Label, value: ByteValue },
    Instruction(Intel8080Instruction),
    LabelDefinition(Label),
    OrgStatement(WordValue),
    WordDefinition { label: Label, value: WordValue },
}

mod lexer;
mod parser;
mod assembler;
pub use assembler::Assembler;
pub use lexer::Lexer;
pub use parser::Parser;