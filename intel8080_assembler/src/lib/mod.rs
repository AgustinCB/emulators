#[macro_use]
extern crate failure;
extern crate intel8080cpu;

use intel8080cpu::Location;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LabelExpression(String);

#[derive(Debug, Fail)]
pub enum AssemblerError {
    #[fail(display = "Unexpected character: {} at line {}", c, line)]
    UnexpectedCharacter { c: char, line: usize },
    #[fail(display = "Expecting {:?}, got {:?} ar line {}", expected, got, line)]
    ExpectingToken {
        expected: AssemblerTokenType,
        got: Option<AssemblerTokenType>,
        line: usize,
    },
    #[fail(display = "Expecting number, got {:?} at line {}", got, line)]
    ExpectingNumber {
        got: Option<AssemblerTokenType>,
        line: usize,
    },
    #[fail(display = "Expecting number, got {:?} at line {}", got, line)]
    ExpectingOperation {
        got: Option<AssemblerTokenType>,
        line: usize,
    },
    #[fail(display = "Expecting single character at line {}", line)]
    ExpectingCharacter { line: usize },
    #[fail(display = "Expecting single quote at line {}", line)]
    ExpectingSingleQuote { line: usize },
    #[fail(display = "Invalid argument for instruction at line {}", line)]
    InvalidInstructionArgument { line: usize },
    #[fail(display = "Invalid operation token at line {}.", line)]
    InvalidOperationToken { line: usize },
    #[fail(display = "Label doesn't exist at line {}.", line)]
    LabelDoesntExist { line: usize },
    #[fail(display = "THERE IS SOMETHING VERY WRONG AT LINE {} DUDE", line)]
    UndefinedError { line: usize },
    #[fail(display = "Unexpected end of expression at line {}", line)]
    UnexpectedEndOfExpression { line: usize },
    #[fail(display = "Label {:?} wasn't declared", label)]
    LabelNotFound { label: LabelExpression },
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
pub enum AssemblerTokenType {
    And,
    Char(char),
    Colon,
    Comma,
    DataStore(Location),
    Db,
    Div,
    Dollar,
    Dw,
    InstructionCode(InstructionCode),
    LabelToken(LabelExpression),
    LeftParen,
    Minus,
    Mod,
    Mult,
    Not,
    Or,
    Org,
    Plus,
    RightParen,
    Shl,
    Shr,
    TwoWord(u16),
    Xor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssemblerToken {
    pub token_type: AssemblerTokenType,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TwoWordExpression {
    Char(char),
    Dollar,
    Literal(u16),
    Label(LabelExpression),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OperationExpression {
    And(Box<OperationExpression>, Box<OperationExpression>),
    Div(Box<OperationExpression>, Box<OperationExpression>),
    Group(Box<OperationExpression>),
    Mod(Box<OperationExpression>, Box<OperationExpression>),
    Mult(Box<OperationExpression>, Box<OperationExpression>),
    Not(Box<OperationExpression>),
    Operand(TwoWordExpression),
    Or(Box<OperationExpression>, Box<OperationExpression>),
    Shl(Box<OperationExpression>, Box<OperationExpression>),
    Shr(Box<OperationExpression>, Box<OperationExpression>),
    Sub(Box<OperationExpression>, Box<OperationExpression>),
    Sum(Box<OperationExpression>, Box<OperationExpression>),
    Xor(Box<OperationExpression>, Box<OperationExpression>),
}

#[derive(Clone, Debug)]
pub enum InstructionArgument {
    TwoWord(OperationExpression),
    DataStore(Location),
    Word(OperationExpression),
}

impl From<OperationExpression> for InstructionArgument {
    #[inline]
    fn from(op: OperationExpression) -> InstructionArgument {
        InstructionArgument::TwoWord(op)
    }
}

impl From<u8> for InstructionArgument {
    #[inline]
    fn from(byte: u8) -> InstructionArgument {
        InstructionArgument::TwoWord(OperationExpression::Operand(TwoWordExpression::Literal(
            u16::from(byte),
        )))
    }
}

impl From<u16> for InstructionArgument {
    #[inline]
    fn from(two_word: u16) -> InstructionArgument {
        InstructionArgument::TwoWord(OperationExpression::Operand(TwoWordExpression::Literal(
            two_word,
        )))
    }
}

#[derive(Debug)]
pub struct Instruction(
    InstructionCode,
    Option<InstructionArgument>,
    Option<InstructionArgument>,
);

pub enum Statement {
    WordDefinitionStatement(LabelExpression, OperationExpression),
    InstructionExprStmt(Instruction),
    LabelDefinitionStatement(LabelExpression),
    OrgStatement(u16),
    TwoWordDefinitionStatement(LabelExpression, OperationExpression),
}

mod assembler;
mod lexer;
mod parser;
pub use assembler::Assembler;
pub use lexer::Lexer;
pub use parser::Parser;
