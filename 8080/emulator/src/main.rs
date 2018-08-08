extern crate disassembler_8080;
extern crate emulator_space_invaders;

use emulator_space_invaders::cpu::{Cpu, ROM_MEMORY_LIMIT};
use disassembler_8080::Instruction;
use std::env::args;
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

struct Timer {
    last_trigger: usize,
    last_check: usize,
    interval: f64,
}

impl Timer {
    pub(crate) fn new(interval: f64) -> Timer {
        let ms = Timer::get_millis();
        Timer {
            interval,
            last_check: ms,
            last_trigger: ms,
        }
    }

    pub(crate) fn update_last_check(&mut self) {
        self.last_check =  Timer::get_millis();
    }

    pub(crate) fn should_trigger(&mut self) -> bool {
        let ms = Timer::get_millis();
        let should = (ms as f64 - self.last_trigger as f64) > self.interval;
        if should {
            self.last_trigger = ms;
        }
        should
    }

    fn get_millis() -> usize {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        (since_the_epoch.as_secs() * 1000) as usize +
            since_the_epoch.subsec_nanos() as usize / 1_000_000
    }
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

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!("Usage: disassembler-8080 [file]")
    }
    let memory = read_file(&args[1]).unwrap();
    start_game(memory);
}
