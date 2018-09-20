use std::iter::{IntoIterator, Peekable};
use std::vec::IntoIter;
use super::{AssemblerToken, Expression};

struct Parser {
    source: Peekable<IntoIter<AssemblerToken>>,
    tokens: Vec<Expression>,
}

impl Parser {
    pub fn new(source: Vec<AssemblerToken>) -> Parser {
        Parser {
            source: source.into_iter().peekable(),
            tokens: Vec::new(),
        }
    }
}