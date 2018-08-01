extern crate disassembler_8080;

use disassembler_8080::Instruction;
use std::cmp::min;
use std::env::args;
use std::fs::File;
use std::io::Read;

fn get_instructions(bytes: Vec<u8>) -> Vec<(u16, Instruction)> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut pass = 0;
    let mut pc: u16 = 0;
    for index in 0..bytes.len() {
        if pass == 0 {
            let i = Instruction::from_bytes(&bytes[index..min(index+3, bytes.len())]);
            let instruction_size = i.size();
            pass = instruction_size - 1;
            result.push((pc, i));
            pc += instruction_size as u16;
        } else {
            pass -= 1;
        }
    }
    result
}

fn read_file(file_name: &str) -> std::io::Result<Vec<u8>> {
    let metadata = std::fs::metadata(file_name)?;
    let mut f = File::open(file_name)?;
    // this may blow up memory if the file is big enough
    // TODO: streams???
    let mut bytes = vec![0; metadata.len() as usize];
    f.read(&mut bytes[..])?;
    Ok(bytes)
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!("Usage: disassembler-8080 [file]")
    }
    let bytes = read_file(&args[1]).unwrap();
    let instructions = get_instructions(bytes);
    for (pc,instruction) in &instructions {
        println!("{:04x} {}", pc, instruction.to_string());
    }
}
