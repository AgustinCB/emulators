extern crate cpu;
#[macro_use] extern crate failure;
extern crate intel8080cpu;
extern crate mos6502cpu;

use cpu::Instruction;
use failure::Error;
use mos6502cpu::Mos6502Instruction;
use intel8080cpu::Intel8080Instruction;
use std::cmp::min;
use std::env::args;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Fail)]
enum DisassemblerError {
    #[fail(display = "unimplemented cpu: {}", name)]
    InvalidCpu { name: String },
}

// This is an arbitrarily chosen number. We either need RFC 2000 or something else that I dunno yet
const ROM_MEMORY_LIMIT: usize = 0x10000;

const USAGE: &'static str = "Usage: disassembler [cpu] [file]

Disassemble a binary file for an old cpu. So far, supports only:

- mos6502
- intel8080";

fn get_instructions_for_cpu(cpu: &str, bytes: [u8; ROM_MEMORY_LIMIT])
    -> Result<Vec<(u16, Box<ToString>)>, Error> {
    match cpu {
        "mos6502" => get_instructions::<Mos6502Instruction>(bytes),
        "intel8080" => get_instructions::<Intel8080Instruction>(bytes),
        _ => Err(Error::from(DisassemblerError::InvalidCpu { name: String::from(cpu) })),
    }
}

fn get_instructions<I: 'static + Instruction + ToString + From<Vec<u8>>>(bytes: [u8; ROM_MEMORY_LIMIT])
    -> Result<Vec<(u16, Box<ToString>)>, Error> {
    let mut result: Vec<(u16, Box<ToString>)> = Vec::with_capacity(bytes.len());
    let mut pass = 0;
    let mut pc: usize = 0;
    for index in 0..bytes.len() {
        if pass == 0 {
            let i =
                I::from(
                    bytes[index..min(index+3, bytes.len())].to_vec());
            let instruction_size = i.size()?;
            pass = instruction_size - 1;
            result.push((pc as u16, Box::new(i)));
            pc += instruction_size as usize;
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

fn disassemble(cpu: &str, memory: [u8; ROM_MEMORY_LIMIT]) -> Result<(), Error> {
    let instructions = get_instructions_for_cpu(cpu,memory)?;
    for (pc,instruction) in &instructions {
        println!("{:04x} {}", pc, instruction.to_string());
    };
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        panic!(USAGE);
    }

    let memory = read_file(&args[2]).unwrap();
    let cpu = &args[1];
    disassemble(cpu, memory).unwrap();
}
