extern crate emulator_8080;

use emulator_8080::cpu::{Executable, State};
use std::env::args;
use std::fs::File;
use std::io::Read;

fn read_file(file_name: &str) -> std::io::Result<[u8; 8192]> {
    let metadata = std::fs::metadata(file_name)?;
    let mut f = File::open(file_name)?;
    // this may blow up memory if the file is big enough
    // TODO: streams???
    let mut bytes = vec![0; metadata.len() as usize];
    f.read(&mut bytes[..])?;
    let mut memory = [0; 8192];
    memory.copy_from_slice(&bytes[..bytes.len()]);
    Ok(memory)
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!("Usage: disassembler-8080 [file]")
    }
    let memory = read_file(&args[1]).unwrap();
    let mut state = State::new(memory);
    while !state.is_done() {
        state.execute();
    }
}
