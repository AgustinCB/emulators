#[macro_use] extern crate failure;
extern crate intel8080cpu;

use intel8080cpu::{Intel8080Instruction, Location};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LabelExpression(String);

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
    #[fail(display = "Invalid operation token.")]
    InvalidOperationToken,
    #[fail(display = "Label doesn't exist.")]
    LabelDoesntExist,
    #[fail(display = "THERE IS SOMETHING VERY WRONG DUDE")]
    UndefinedError,
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
    LabelToken(LabelExpression),
    Minus,
    Org,
    Word(u16),
    Plus,
}

pub enum ByteExpression {
    Literal(u8),
    Label(LabelExpression),
}

pub enum ByteValue {
    Operand(ByteExpression),
    Sum(ByteExpression, ByteExpression),
    Rest(ByteExpression, ByteExpression),
}

pub enum WordExpression {
    Literal(u16),
    Label(LabelExpression),
}

pub enum WordValue {
    Operand(WordExpression),
    Sum(WordExpression, WordExpression),
    Rest(WordExpression, WordExpression),
}

pub enum Statement {
    ByteDefinitionStatement { label: LabelExpression, value: ByteValue },
    InstructionExprStmt(Intel8080Instruction),
    LabelDefinitionStatement(LabelExpression),
    OrgStatement(WordValue),
    WordDefinitionStatement { label: LabelExpression, value: WordValue },
}

mod lexer;
mod parser;
mod assembler;
pub use assembler::Assembler;
pub use lexer::Lexer;
pub use parser::Parser;