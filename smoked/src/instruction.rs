use log::warn;

#[derive(Clone, Debug, PartialEq)]
pub enum InstructionType {
    Return,
    Constant(usize),
    Nil,
    True,
    False,
    Plus,
    Minus,
    Mult,
    Div,
    Not,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Noop,
    StringConcat,
    Syscall,
    SetGlobal(usize),
    GetGlobal(usize),
    SetLocal(usize),
    GetLocal(usize),
    JmpIfFalse(usize),
    Jmp(usize),
    Loop(usize),
    Call,
    ArrayAlloc,
    ArrayGet,
    ArraySet,
    MultiArraySet,
    ObjectAlloc,
    ObjectGet,
    ObjectSet,
    And,
    Or,
    Abs,
    Push,
    Pop,
    RepeatedArraySet,
    Strlen,
    Swap,
    ToStr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub location: usize,
}

impl Instruction {
    pub fn size(&self) -> usize {
        match &self.instruction_type {
            InstructionType::Constant(_) | InstructionType::SetGlobal(_) | InstructionType::GetGlobal(_) |
            InstructionType::SetLocal(_) | InstructionType::GetLocal(_) | InstructionType::Jmp(_) |
            InstructionType::JmpIfFalse(_) | InstructionType::Loop(_) => 17,
            _ => 9,
        }
    }
}

#[inline]
fn create_instruction(instruction_type: InstructionType, bytes: &[u8]) -> Instruction {
    Instruction {
        instruction_type,
        location: usize::from_le_bytes(
            [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]],
        ),
    }
}

impl Into<Vec<u8>> for Instruction {
    fn into(self) -> Vec<u8> {
        let mut bytes = vec![];
        match self.instruction_type {
            InstructionType::Return => bytes.push(0),
            InstructionType::Constant(b) => {
                bytes.push(1);
                bytes.extend_from_slice(&b.to_le_bytes());
            },
            InstructionType::Plus => bytes.push(2),
            InstructionType::Minus => bytes.push(3),
            InstructionType::Mult => bytes.push(4),
            InstructionType::Div => bytes.push(5),
            InstructionType::Noop => bytes.push(255),
            InstructionType::Nil => bytes.push(6),
            InstructionType::True => bytes.push(7),
            InstructionType::False => bytes.push(8),
            InstructionType::Not => bytes.push(9),
            InstructionType::Equal => bytes.push(10),
            InstructionType::NotEqual => bytes.push(11),
            InstructionType::Less => bytes.push(14),
            InstructionType::LessEqual => bytes.push(15),
            InstructionType::Greater => bytes.push(12),
            InstructionType::GreaterEqual => bytes.push(13),
            InstructionType::StringConcat => bytes.push(16),
            InstructionType::Syscall => bytes.push(17),
            InstructionType::GetGlobal(g) => {
                bytes.push(18);
                bytes.extend_from_slice(&g.to_le_bytes());
            },
            InstructionType::SetGlobal(g) => {
                bytes.push(19);
                bytes.extend_from_slice(&g.to_le_bytes());
            },
            InstructionType::GetLocal(g) => {
                bytes.push(20);
                bytes.extend_from_slice(&g.to_le_bytes());
            },
            InstructionType::SetLocal(g) => {
                bytes.push(21);
                bytes.extend_from_slice(&g.to_le_bytes());
            },
            InstructionType::JmpIfFalse(offset) => {
                bytes.push(22);
                bytes.extend_from_slice(&offset.to_le_bytes());
            },
            InstructionType::Jmp(offset) => {
                bytes.push(23);
                bytes.extend_from_slice(&offset.to_le_bytes());
            },
            InstructionType::Loop(offset) => {
                bytes.push(24);
                bytes.extend_from_slice(&offset.to_le_bytes());
            },
            InstructionType::Call => bytes.push(25),
            InstructionType::ArrayAlloc => bytes.push(26),
            InstructionType::ArrayGet => bytes.push(27),
            InstructionType::ArraySet => bytes.push(29),
            InstructionType::ObjectAlloc => bytes.push(29),
            InstructionType::ObjectGet => bytes.push(30),
            InstructionType::ObjectSet => bytes.push(31),
            InstructionType::And => bytes.push(32),
            InstructionType::Or => bytes.push(33),
            InstructionType::Abs => bytes.push(34),
            InstructionType::MultiArraySet => bytes.push(35),
            InstructionType::Push => bytes.push(36),
            InstructionType::Pop => bytes.push(37),
            InstructionType::RepeatedArraySet => bytes.push(38),
            InstructionType::Strlen => bytes.push(39),
            InstructionType::Swap => bytes.push(40),
            InstructionType::ToStr => bytes.push(41),
        }
        bytes.extend_from_slice(&self.location.to_le_bytes());
        bytes
    }
}

impl From<&[u8]> for Instruction {
    #[inline]
    fn from(bytes: &[u8]) -> Instruction {
        match bytes[0] {
            0 => create_instruction(InstructionType::Return, &bytes[1..]),
            1 => create_instruction(InstructionType::Constant(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            2 => create_instruction(InstructionType::Plus, &bytes[1..]),
            3 => create_instruction(InstructionType::Minus, &bytes[1..]),
            4 => create_instruction(InstructionType::Mult, &bytes[1..]),
            5 => create_instruction(InstructionType::Div, &bytes[1..]),
            6 => create_instruction(InstructionType::Nil, &bytes[1..]),
            7 => create_instruction(InstructionType::True, &bytes[1..]),
            8 => create_instruction(InstructionType::False, &bytes[1..]),
            9 => create_instruction(InstructionType::Not, &bytes[1..]),
            10 => create_instruction(InstructionType::Equal, &bytes[1..]),
            11 => create_instruction(InstructionType::NotEqual, &bytes[1..]),
            12 => create_instruction(InstructionType::Greater, &bytes[1..]),
            13 => create_instruction(InstructionType::GreaterEqual, &bytes[1..]),
            14 => create_instruction(InstructionType::Less, &bytes[1..]),
            15 => create_instruction(InstructionType::LessEqual, &bytes[1..]),
            16 => create_instruction(InstructionType::StringConcat, &bytes[1..]),
            17 => create_instruction(InstructionType::Syscall, &bytes[1..]),
            18 => create_instruction(InstructionType::GetGlobal(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            19 => create_instruction(InstructionType::SetGlobal(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            20 => create_instruction(InstructionType::GetLocal(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            21 => create_instruction(InstructionType::SetLocal(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            22 => create_instruction(InstructionType::JmpIfFalse(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            23 => create_instruction(InstructionType::Jmp(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            24 => create_instruction(InstructionType::Loop(usize::from_le_bytes(
                [bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]],
            )), &bytes[9..]),
            25 => create_instruction(InstructionType::Call, &bytes[1..]),
            26 => create_instruction(InstructionType::ArrayAlloc, &bytes[1..]),
            27 => create_instruction(InstructionType::ArrayGet, &bytes[1..]),
            28 => create_instruction(InstructionType::ArraySet, &bytes[1..]),
            29 => create_instruction(InstructionType::ObjectAlloc, &bytes[1..]),
            30 => create_instruction(InstructionType::ObjectGet, &bytes[1..]),
            31 => create_instruction(InstructionType::ObjectSet, &bytes[1..]),
            32 => create_instruction(InstructionType::And, &bytes[1..]),
            33 => create_instruction(InstructionType::Or, &bytes[1..]),
            34 => create_instruction(InstructionType::Abs, &bytes[1..]),
            35 => create_instruction(InstructionType::MultiArraySet, &bytes[1..]),
            36 => create_instruction(InstructionType::Push, &bytes[1..]),
            37 => create_instruction(InstructionType::Pop, &bytes[1..]),
            38 => create_instruction(InstructionType::RepeatedArraySet, &bytes[1..]),
            39 => create_instruction(InstructionType::Strlen, &bytes[1..]),
            40 => create_instruction(InstructionType::Swap, &bytes[1..]),
            41 => create_instruction(InstructionType::ToStr, &bytes[1..]),
            255 => create_instruction(InstructionType::Noop, &bytes[1..]),
            _ => {
                warn!("Invalid instruction");
                create_instruction(InstructionType::Noop, &bytes[1..])
            },
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match &self.instruction_type {
            InstructionType::Return => "RETURN".to_owned(),
            InstructionType::Constant(b) => format!("CONSTANT {}", b),
            InstructionType::Plus => "PLUS".to_owned(),
            InstructionType::Minus => "MINUS".to_owned(),
            InstructionType::Mult => "MULT".to_owned(),
            InstructionType::Div => "DIV".to_owned(),
            InstructionType::Noop => "NOOP".to_owned(),
            InstructionType::Nil => "NIL".to_owned(),
            InstructionType::True => "TRUE".to_owned(),
            InstructionType::False => "FALSE".to_owned(),
            InstructionType::Not => "NOT".to_owned(),
            InstructionType::Equal => "EQUAL".to_owned(),
            InstructionType::NotEqual => "NOT_EQUAL".to_owned(),
            InstructionType::Less => "LESS".to_owned(),
            InstructionType::LessEqual => "LESS_EQUAL".to_owned(),
            InstructionType::Greater => "GREATER".to_owned(),
            InstructionType::GreaterEqual => "GREATER_EQUAL".to_owned(),
            InstructionType::StringConcat => "STRING_CONCAT".to_owned(),
            InstructionType::Syscall => "SYSCALL".to_owned(),
            InstructionType::GetGlobal(g) => format!("GET_GLOBAL {}", g),
            InstructionType::SetGlobal(g) => format!("SET_GLOBAL {}", g),
            InstructionType::GetLocal(g) => format!("GET_LOCAL {}", g),
            InstructionType::SetLocal(g) => format!("SET_LOCAL {}", g),
            InstructionType::JmpIfFalse(offset) => format!("JMP_IF_FALSE {}", offset),
            InstructionType::Jmp(offset) => format!("JMP {}", offset),
            InstructionType::Loop(offset) => format!("LOOP {}", offset),
            InstructionType::Call => "CALL".to_owned(),
            InstructionType::ArrayAlloc => "ARRAY_ALLOC".to_owned(),
            InstructionType::ArrayGet => "ARRAY_GET".to_owned(),
            InstructionType::ArraySet => "ARRAY_SET".to_owned(),
            InstructionType::MultiArraySet => "MULTI_ARRAY_SET".to_owned(),
            InstructionType::ObjectAlloc => "OBJECT_ALLOC".to_owned(),
            InstructionType::ObjectGet => "OBJECT_GET".to_owned(),
            InstructionType::ObjectSet => "OBJECT_SET".to_owned(),
            InstructionType::And => "AND".to_owned(),
            InstructionType::Or => "OR".to_owned(),
            InstructionType::Abs => "ABS".to_owned(),
            InstructionType::Push => "PUSH".to_owned(),
            InstructionType::Pop => "POP".to_owned(),
            InstructionType::RepeatedArraySet => "REPEATED_ARRAY_SET".to_owned(),
            InstructionType::Strlen => "STRLEN".to_owned(),
            InstructionType::Swap => "SWAP".to_owned(),
            InstructionType::ToStr => "TO_STR".to_owned(),
        }
    }
}
