extern crate intel8080cpu;

use intel8080cpu::{Location, Intel8080Instruction};

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