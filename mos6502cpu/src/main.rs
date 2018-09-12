extern crate mos6502cpu;
extern crate failure;

use failure::Error;
use mos6502cpu::{AVAILABLE_MEMORY, Cpu, Mos6502Cpu};
use std::env::args;
use std::fs::File;
use std::io::Read;

const USAGE: &'static str = "Usage: mos6502cpu [file] [starting address]

Runs [file], a MOS 6502 compatible binary file, in the emulator.

It starts at [starting address].";

fn read_file(file_name: &str) -> std::io::Result<[u8; AVAILABLE_MEMORY]> {
    let mut f = File::open(file_name)?;
    let mut memory = [0; AVAILABLE_MEMORY];
    f.read(&mut memory)?;
    Ok(memory)
}

fn test(mut memory: [u8; AVAILABLE_MEMORY], starting_address: u16) -> Result<(), Error> {
    let mut cpu = Mos6502Cpu::new(&mut memory);
    cpu.set_pc(starting_address);
    while !cpu.is_done() {
        cpu.execute()?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        panic!(USAGE);
    }
    let memory = read_file(&args[1]).unwrap();
    let starting_address = args[2].parse::<u16>().unwrap();
    test(memory, starting_address).unwrap();
}