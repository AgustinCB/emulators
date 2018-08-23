extern crate intel8080cpu;
extern crate emulator_space_invaders;
extern crate failure;

use intel8080cpu::*;
use emulator_space_invaders::console::Console;
use failure::Error;
use std::env::args;
use std::cmp::min;
use std::fs::File;
use std::io::Read;

const USAGE: &'static str = "Usage: disassembler-8080 [game|test] [file]

If running either test, [file] should be a hex file with Intel 8080 instructions.

When selecting the mode game, [file] should be a folder that contains the following content:

./rom # The rom of the game
./0.wav ... 9.wav # The audio files of the game";

struct PrintScreen;

impl Printer for PrintScreen {
    fn print(&mut self, bytes: &[u8]) {
        println!("{}", String::from_utf8_lossy(bytes));
    }
}

fn read_file(file_name: &str) -> std::io::Result<[u8; ROM_MEMORY_LIMIT]> {
    let mut f = File::open(file_name)?;
    // this may blow up memory if the file is big enough
    // TODO: streams???
    let mut memory = [0; ROM_MEMORY_LIMIT];
    f.read(&mut memory)?;
    Ok(memory)
}

fn start_game(folder: &str) -> Result<(), Error> {
    let rom_location = format!("{}/rom", folder);
    let memory = read_file(&rom_location)?;
    let mut console = Console::new(memory, folder)?;
    console.start().map_err(|e| Error::from(e))
}

fn test(memory: [u8; ROM_MEMORY_LIMIT]) -> Result<(), Error> {
    let screen = &mut (PrintScreen {});
    let mut cpu = Intel8080Cpu::new_cp_m_compatible(memory, screen);

    while !cpu.is_done() {
        cpu.execute()?;
    }
    Ok(())
}

fn handle_result(r: Result<(), Error>) {
    match r {
        Ok(()) => {},
        Err(err) => panic!(err),
    };
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        panic!(USAGE);
    }

    if args[1] == "game" {
        handle_result(start_game(&args[2]));
    } else if args[1] == "test" {
        let memory = read_file(&args[2]).unwrap();
        handle_result(test(memory));
    } else {
        panic!(USAGE);
    }
}
