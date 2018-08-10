use std::time::Duration;
use std::thread::sleep;
use super::cpu::{Cpu, Instruction, ROM_MEMORY_LIMIT};
use super::io_devices::*;
use super::timer::Timer;

const SCREEN_INTERRUPTIONS_INTERVAL: f64 = 60.0/1.0*1000.0;

pub struct Console<'a> {
    cpu: Cpu<'a>,
    timer: Timer,
    next_interruption: u8,
    cycles_left: usize,
}

impl<'a> Console<'a> {
    pub fn new<'b>(memory: [u8; ROM_MEMORY_LIMIT]) -> Console<'b> {
        let mut cpu = Cpu::new(memory);
        let timer = Timer::new(SCREEN_INTERRUPTIONS_INTERVAL);
        let shift_writer = ExternalShiftWriter::new();
        let offset_writer = ExternalShiftOffsetWriter::new();
        let shift_reader = ExternalShiftReader::new(&shift_writer, &offset_writer);

        cpu.add_input_device(0, Box::new(DummyInputDevice { value: 1 }));
        cpu.add_input_device(1, Box::new(KeypadInput { }));
        cpu.add_input_device(2, Box::new(DummyInputDevice { value: 1 }));
        cpu.add_input_device(3, Box::new(shift_reader));
        cpu.add_output_device(2, Box::new(offset_writer));
        cpu.add_output_device(3, Box::new(DummyOutputDevice{}));
        cpu.add_output_device(4, Box::new(shift_writer));
        cpu.add_output_device(5, Box::new(DummyOutputDevice{}));
        cpu.add_output_device(6, Box::new(DummyOutputDevice{}));

        Console {
            cpu,
            timer,
            next_interruption: 1,
            cycles_left: 0,
        }
    }

    pub fn start(&mut self) {
        while !self.cpu.is_done() {
            let elapsed = self.timer.update_last_check();
            if self.timer.should_trigger() {
                self.cpu.execute_instruction(Instruction::Rst {
                    value: self.next_interruption
                });
                self.next_interruption = if self.next_interruption == 1 {
                    2
                } else {
                    1
                };
            }
            let mut cycles_to_run = elapsed * 2 + self.cycles_left;
            while cycles_to_run > 0 {
                cycles_to_run -= self.cpu.execute() as usize;
            }
            self.cycles_left = cycles_to_run;
            sleep(Duration::from_millis(10));
        }
    }
}