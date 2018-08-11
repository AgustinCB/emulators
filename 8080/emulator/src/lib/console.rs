use std::time::Duration;
use std::thread::sleep;
use super::cpu::{Cpu, Instruction, ROM_MEMORY_LIMIT};
use super::io_devices::*;
use super::screen::{Screen, TermScreen};
use super::timer::Timer;

const SCREEN_INTERRUPTIONS_INTERVAL: f64 = (1.0/60.0*1000.0)/2.0;

pub struct Console<'a> {
    cpu: Cpu<'a>,
    cycles_left: i64,
    prev_interruption: u8,
    screen: Box<Screen>,
    timer: Timer,
}

impl<'a> Console<'a> {
    pub fn new<'b>(memory: [u8; ROM_MEMORY_LIMIT]) -> Console<'b> {
        let mut cpu = Cpu::new(memory);
        let timer = Timer::new(SCREEN_INTERRUPTIONS_INTERVAL);
        let screen = Box::new(TermScreen::new(&cpu.memory));
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
            cycles_left: 0,
            prev_interruption: 2,
            screen,
            timer,
        }
    }

    pub fn start(&mut self) {
        self.timer.reset();
        while !self.cpu.is_done() {
            let elapsed = self.timer.update_last_check();
            if self.timer.should_trigger() && self.cpu.interruptions_enabled {
                self.prev_interruption = if self.prev_interruption == 1 {
                    self.screen.on_full_screen();
                    2
                } else {
                    self.screen.on_mid_screen();
                    1
                };
                self.cpu.execute_instruction(Instruction::Rst {
                    value: self.prev_interruption
                });
            }
            let mut cycles_to_run = (elapsed * 2) as i64 + self.cycles_left;
            while cycles_to_run > 0 {
                cycles_to_run -= self.cpu.execute() as i64;
            }
            self.cycles_left = cycles_to_run;
            sleep(Duration::from_millis(1));
        }
    }
}