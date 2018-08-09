extern crate emulator_space_invaders;

use emulator_space_invaders::cpu::{Cpu, Instruction, ROM_MEMORY_LIMIT};
use emulator_space_invaders::timer::Timer;
use std::env::args;
use std::fs::File;
use std::io::Read;

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

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!("Usage: disassembler-8080 [file]")
    }
    let memory = read_file(&args[1]).unwrap();
    start_game(memory);
}
