extern crate emulator_space_invaders;

use emulator_space_invaders::cpu::{Cpu, Instruction, ROM_MEMORY_LIMIT};
use emulator_space_invaders::timer::Timer;
use std::env::args;
use std::cmp::min;
use std::fs::File;
use std::io::Read;

const USAGE: &'static str = "Usage: disassembler-8080 [run|disassemble] [file]";

fn get_instructions(bytes: [u8; ROM_MEMORY_LIMIT]) -> Vec<(u16, Instruction)> {
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

fn read_file(file_name: &str) -> std::io::Result<[u8; ROM_MEMORY_LIMIT]> {
    let metadata = std::fs::metadata(file_name)?;
    let mut f = File::open(file_name)?;
    // this may blow up memory if the file is big enough
    // TODO: streams???
    let mut bytes = vec![0; metadata.len() as usize];
    f.read(&mut bytes[..])?;
    let mut memory = [0; ROM_MEMORY_LIMIT];
    memory.copy_from_slice(&bytes[..bytes.len()]);
    Ok(memory)
}

fn start_game(memory: [u8; ROM_MEMORY_LIMIT]) {
    let mut cpu = Cpu::new(memory);
    let mut next_interrupt: u8 = 1;
    let mut timer = Timer::new(60.0/1.0*1000.0);

    while !cpu.is_done() {
        timer.update_last_check();
        if timer.should_trigger() {
            cpu.execute_instruction(Instruction::Rst { value: next_interrupt });
            next_interrupt = if next_interrupt == 1 {
                2
            } else {
                1
            };
        }
        cpu.execute();
    }
}

fn disassemble(memory: [u8; ROM_MEMORY_LIMIT]) {
    let instructions = get_instructions(memory);
    for (pc,instruction) in &instructions {
        println!("{:04x} {}", pc, instruction.to_string());
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        panic!(USAGE);
    }
    let memory = read_file(&args[2]).unwrap();

    if args[2] == "run" {
        start_game(memory);
    } else if args[2] == "disassemble" {
        disassemble(memory);
    } else {
        panic!(USAGE);
    }
}
