use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Peekable;
use std::str::FromStr;
use std::mem::size_of;
use smoked::cpu::{VALUE_SIZE, USIZE_SIZE};

const USAGE: &str = "Usage: smoke-assembler [input file] [output file]";

#[derive(Debug)]
struct Config {
    input_file: Option<String>,
    output_file: Option<String>,
}

fn parse_config<I: Iterator<Item=String>>(mut strings: I) -> Config {
    let mut configuration = Config {
        input_file: None,
        output_file: None,
    };
    strings.next();
    while let Some(next) = strings.next() {
        match next.as_str() {
            s if configuration.input_file.is_none() => {
                configuration.input_file = Some(s.to_owned());
            }
            s if configuration.output_file.is_none() => {
                configuration.output_file = Some(s.to_owned());
            }
            _ => panic!(USAGE),
        }
    }
    configuration
}

#[derive(Clone, Debug)]
enum Constant<'a> {
    Nil,
    Integer(i64),
    Float(f32),
    Bool(bool),
    String(&'a str),
    Function {
        ip: usize,
        arity: usize,
    },
    Array {
        capacity: usize,
    },
    Object {
        capacity: usize,
    },
}

#[derive(Clone, Debug)]
enum Token<'a> {
    Data,
    Constant(Constant<'a>),
    Code,
    Instruction(&'a str),
    Number(usize),
}

fn take_while<'a, P: Fn(char) -> bool>(chars: &[char], f: P) -> usize {
    let mut result = 0usize;
    for c in chars {
        if f(*c) {
            result += 1;
        } else {
            break;
        }
    }
    result
}

macro_rules! get_str {
    ($var: ident, $chars: ident, $index: ident, $content: ident, $predicate: expr) => {
        let offset = take_while(&$chars[$index..], $predicate);
        let from = $index;
        $index += offset;
        let $var = &$content[from..$index];
    }
}

macro_rules! expect_quotes {
    ($chars: ident, $index: ident) => {
        if $chars[$index] != '"' {
            panic!("Expected quotes");
        }
        $index += 1;
    }
}

macro_rules! expect_whitespace {
    ($chars: ident, $index: ident) => {
        if !$chars[$index].is_whitespace() {
            panic!("Expected whitespace");
        }
        $index += 1;
    }
}

fn lexer(content: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let chars: Vec<char> = content.chars().peekable().collect();
    let mut index = 0usize;
    while index < chars.len() {
        let c = chars[index];
        index += 1;
        match c {
            '.' => {
                get_str!(pred, chars, index, content, |c| !c.is_whitespace());
                match pred {
                    "data" => {
                        tokens.push(Token::Data);
                    }
                    "code" => {
                        tokens.push(Token::Code);
                    }
                    p => panic!("Invalid predicate {}", p),
                }
            }
            c if !c.is_whitespace() => {
                let instruction_start = index;
                get_str!(suffix, chars, index, content, |c| !c.is_whitespace());
                let keyword = format!("{}{}", c, suffix);
                expect_whitespace!(chars, index);
                match keyword.to_lowercase().as_str() {
                    "nil" => tokens.push(Token::Constant(Constant::Nil)),
                    "integer" => {
                        get_str!(number_string, chars, index, content, |c| !c.is_whitespace());
                        let number = i64::from_str(number_string).unwrap();
                        tokens.push(Token::Constant(Constant::Integer(number)));
                    }
                    "float" => {
                        get_str!(number_string, chars, index, content, |c| !c.is_whitespace());
                        let number = f32::from_str(number_string).unwrap();
                        tokens.push(Token::Constant(Constant::Float(number)));
                    }
                    "bool" => {
                        get_str!(bool, chars, index, content, |c| !c.is_whitespace());
                        let value = bool::from_str(bool).unwrap();
                        tokens.push(Token::Constant(Constant::Bool(value)));
                    }
                    "string" => {
                        expect_quotes!(chars, index);
                        get_str!(value, chars, index, content, |c| c != '"');
                        expect_quotes!(chars, index);
                        tokens.push(Token::Constant(Constant::String(value)));
                    }
                    "function" => {
                        get_str!(ip_string, chars, index, content, |c| !c.is_whitespace());
                        let ip = usize::from_str(ip_string).unwrap();
                        expect_whitespace!(chars, index);
                        get_str!(arity_string, chars, index, content, |c| !c.is_whitespace());
                        let arity = usize::from_str(arity_string).unwrap();
                        tokens.push(Token::Constant(Constant::Function { ip, arity }));
                    }
                    "array" => {
                        get_str!(capacity_string, chars, index, content, |c| !c.is_whitespace());
                        let capacity = usize::from_str(capacity_string).unwrap();
                        tokens.push(Token::Constant(Constant::Array { capacity }));
                    }
                    "object" => {
                        get_str!(capacity_string, chars, index, content, |c| !c.is_whitespace());
                        let capacity = usize::from_str(capacity_string).unwrap();
                        tokens.push(Token::Constant(Constant::Object { capacity }));
                    }
                    instruction => {
                        match instruction.parse::<usize>() {
                            Ok(number) => tokens.push(Token::Number(number)),
                            Err(_) => tokens.push(Token::Instruction(&content[instruction_start-1..index-1])),
                        }
                    },
                }
            },
            s if s.is_whitespace() => {},
            _ => panic!("Unexpected space")
        }
    }
    tokens
}

macro_rules! serialize_type {
    ($bytes: ident, $value: expr, $type: ident) => {
        let p: &[u8] = unsafe {
            std::slice::from_raw_parts(&$value as *const $type as *const u8, size_of::<$type>())
        };
        $bytes.extend_from_slice(p);
    }
}

fn parse_constants<'a, I: Iterator<Item=Token<'a>>>(lexems: &mut Peekable<I>) -> (Vec<u8>, Vec<u8>) {
    let mut memory = vec![];
    let mut bytes = vec![];
    if let Some(Token::Data) = lexems.peek().cloned() {
        lexems.next();
        while let Some(Token::Constant(constant)) = lexems.peek().cloned() {
            lexems.next();
            match constant {
                Constant::Nil => bytes.push(0),
                Constant::Integer(i) => {
                    bytes.push(1);
                    serialize_type!(bytes, i, i64);
                },
                Constant::Float(f) => {
                    bytes.push(2);
                    serialize_type!(bytes, f, f32);
                },
                Constant::Bool(b) => {
                    bytes.push(3);
                    bytes.push(if b { 1 } else { 0 });
                }
                Constant::String(s) => {
                    bytes.push(4);
                    serialize_type!(bytes, s.len(), usize);
                    memory.extend_from_slice(s.as_bytes());
                }
                Constant::Function { ip, arity } => {
                    bytes.push(5);
                    serialize_type!(bytes, ip, usize);
                    serialize_type!(bytes, arity, usize);
                }
                Constant::Array { capacity } => {
                    bytes.push(6);
                    let capacity = VALUE_SIZE * capacity;
                    serialize_type!(bytes, capacity, usize);
                    for _ in 0..capacity {
                        memory.push(0)
                    }
                }
                Constant::Object { capacity } => {
                    bytes.push(7);
                    serialize_type!(memory, capacity, usize);
                    let capacity = (USIZE_SIZE + VALUE_SIZE) * capacity;
                    serialize_type!(bytes, capacity, usize);
                    for _ in 0..capacity {
                        memory.push(0)
                    }
                }
            }
        }
    }
    (memory, bytes)
}

fn parse_instructions<'a, I: Iterator<Item=Token<'a>>>(lexems: &mut Peekable<I>) -> Vec<u8> {
    let mut upcodes = vec![];
    if let Some(Token::Code) = lexems.peek().cloned() {
        lexems.next();
        while let Some(token) = lexems.next() {
            match token {
                Token::Instruction(i) => {
                    match i {
                        "RETURN" => upcodes.push(0),
                        "CONSTANT" => upcodes.push(1),
                        "PLUS" => upcodes.push(2),
                        "MINUS" => upcodes.push(3),
                        "MULT" => upcodes.push(4),
                        "DIV" => upcodes.push(5),
                        "NIL" => upcodes.push(6),
                        "TRUE" => upcodes.push(7),
                        "FALSE" => upcodes.push(8),
                        "NOT" => upcodes.push(9),
                        "EQUAL" => upcodes.push(10),
                        "NOT_EQUAL" => upcodes.push(11),
                        "GREATER" => upcodes.push(12),
                        "GREATER_EQUAL" => upcodes.push(13),
                        "LESS" => upcodes.push(14),
                        "LESS_EQUAL" => upcodes.push(15),
                        "STRING_CONCAT" => upcodes.push(16),
                        "SYSCALL" => upcodes.push(17),
                        "GET_GLOBAL" => upcodes.push(18),
                        "SET_GLOBAL" => upcodes.push(19),
                        "GET_LOCAL" => upcodes.push(20),
                        "SET_LOCAL" => upcodes.push(21),
                        "JMP_IF_FALSE" => upcodes.push(22),
                        "JMP" => upcodes.push(23),
                        "LOOP" => upcodes.push(24),
                        "CALL" => upcodes.push(25),
                        "ARRAY_ALLOC" => upcodes.push(26),
                        "ARRAY_GET" => upcodes.push(27),
                        "ARRAY_SET" => upcodes.push(28),
                        "OBJECT_ALLOC" => upcodes.push(29),
                        "OBJECT_GET" => upcodes.push(30),
                        "OBJECT_SET" => upcodes.push(31),
                        "NOOP" => upcodes.push(255),
                        _ => panic!("Unexpected instruction {}", i),
                    }
                },
                Token::Number(n) => {
                    serialize_type!(upcodes, n, usize);
                },
                t => panic!("Unexpected token, {:?}, only instructions or tokens expected", t),
            }
        }
    }
    upcodes
}

fn main() {
    let conf = parse_config(args());
    let mut input_file: Box<dyn Read> = conf.input_file
        .map::<Box<dyn Read>, _>(|f| { Box::new(File::open(f).unwrap()) })
        .unwrap_or_else(|| Box::new(std::io::stdin()));
    let mut output_file: Box<dyn Write> = conf.output_file
        .map::<Box<dyn Write>, _>(|f| Box::new(File::create(f).unwrap()))
        .unwrap_or_else(|| Box::new(std::io::stdout()));
    let mut content = "".to_owned();
    input_file.read_to_string(&mut content).unwrap();
    let mut lexems = lexer(&content).into_iter().peekable();
    let (memory, constants) = parse_constants(&mut lexems);
    let upcodes = parse_instructions(&mut lexems);
    let mut output = vec![];
    serialize_type!(output, constants.len(), usize);
    serialize_type!(output, memory.len(), usize);
    output.extend_from_slice(&constants);
    output.extend_from_slice(&memory);
    output.extend_from_slice(&upcodes);
    output_file.write_all(&output).unwrap();
}