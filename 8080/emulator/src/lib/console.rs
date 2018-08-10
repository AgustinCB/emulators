use std::cell::Cell;
use super::cpu::{Cpu, ROM_MEMORY_LIMIT};
use super::io_devices::{DummyInputDevice, DummyOutputDevice, ExternalShift};

const NUM_INPUT_DEVICES: usize = 4;
const NUM_OUTPUT_DEVICES: usize = 5;

pub struct Console<'a> {
    pub cpu: Cpu<'a>,
    pub external_shift: Cell<ExternalShift>,
}

impl<'a> Console<'a> {
    pub fn new<'b>(memory: [u8; ROM_MEMORY_LIMIT]) -> Console<'b> {
        let mut cpu = Cpu::new(memory);

        for _ in 1..NUM_INPUT_DEVICES {
            let mut dev = DummyInputDevice { value: 0 };
            cpu.add_input_device(Box::new(dev));
        }

        for _ in 1..NUM_OUTPUT_DEVICES {
            let mut dev = DummyOutputDevice { };
            cpu.add_output_device(Box::new(dev));
        }

        Console {
            cpu,
            external_shift: Cell::new(ExternalShift::new()),
        }
    }
}