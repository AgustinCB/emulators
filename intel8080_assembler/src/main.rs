extern crate intel8080_assembler;

use intel8080_assembler::{Assembler, Lexer, Parser};
use std::env::args;
use std::fs::File;
use std::io::Write;

const USAGE: &str = "Usage: intel8080_assembler [input file] [output file]

Assemble an intel 8080 asm file.";

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        panic!(USAGE);
    }

    let f = File::open(&args[1]).unwrap();
    let lexer = Lexer::new(f);
    let tokens = lexer.scan_tokens().unwrap();
    let parser = Parser::new(tokens);
    let statements = parser.parse_statements().unwrap();
    let assembler = Assembler::new();
    let output = assembler.assemble(statements).unwrap();

    let mut output_file = File::create(&args[2]).unwrap();
    output_file.write_all(&output).unwrap();
}
