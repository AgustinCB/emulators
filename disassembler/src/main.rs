extern crate mos6502cpu;
extern crate failure;

use failure::Error;
use mos6502cpu::{Instruction, Mos6502Instruction};
use std::cmp::min;
use std::env::args;
use std::fs::File;
use std::io::Read;

// This is an arbitrarily chosen number. We either need RFC 2000 or something else that I dunno yet
const ROM_MEMORY_LIMIT: usize = 8192;

const USAGE: &'static str = "Usage: disassembler [file]

Disassemble a binary file for an old cpu. So far, supports only:

- MOS 6502";

fn get_instructions(bytes: [u8; ROM_MEMORY_LIMIT]) -> Result<Vec<(u16, Mos6502Instruction)>, Error> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut pass = 0;
    let mut pc: u16 = 0;
    for index in 0..bytes.len() {
        if pass == 0 {
            let i =
                Mos6502Instruction::from(
                    bytes[index..min(index+3, bytes.len())].to_vec());
            let instruction_size = i.size()?;
            pass = instruction_size - 1;
            result.push((pc, i));
            pc += instruction_size as u16;
        } else {
            pass -= 1;
        }
    }
    Ok(result)
}

fn read_file(file_name: &str) -> std::io::Result<[u8; ROM_MEMORY_LIMIT]> {
    let mut f = File::open(file_name)?;
    let mut memory = [0; ROM_MEMORY_LIMIT];
    f.read(&mut memory)?;
    Ok(memory)
}

fn disassemble(memory: [u8; ROM_MEMORY_LIMIT]) -> Result<(), Error> {
    let instructions = get_instructions(memory)?;
    for (pc,instruction) in &instructions {
        println!("{:04x} {}", pc, instruction.to_string());
    };
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!(USAGE);
    }

    let memory = read_file(&args[1]).unwrap();
    match disassemble(memory) {
        Ok(()) => {},
        Err(err) => panic!(err),
    };
}
