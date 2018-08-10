use super::cpu::{Cpu, ROM_MEMORY_LIMIT};
use super::io_devices::*;

pub struct Console<'a> {
    pub cpu: Cpu<'a>,
}

impl<'a> Console<'a> {
    pub fn new<'b>(memory: [u8; ROM_MEMORY_LIMIT]) -> Console<'b> {
        let mut cpu = Cpu::new(memory);
        let shift_writer = ExternalShiftWriter::new();
        let offset_writer = ExternalShiftOffsetWriter::new();
        let shift_reader = ExternalShiftReader {
            shift_offset: offset_writer.get_shift_offset(),
            shift0: shift_writer.get_shift0(),
            shift1: shift_writer.get_shift1(),
        };

        cpu.add_input_device(0, Box::new(DummyInputDevice { value: 0 }));
        cpu.add_input_device(1, Box::new(DummyInputDevice { value: 0 }));
        cpu.add_input_device(2, Box::new(DummyInputDevice { value: 0 }));
        cpu.add_input_device(3, Box::new(shift_reader));
        cpu.add_output_device(2, Box::new(offset_writer));
        cpu.add_output_device(3, Box::new(DummyOutputDevice{}));
        cpu.add_output_device(4, Box::new(shift_writer));
        cpu.add_output_device(5, Box::new(DummyOutputDevice{}));
        cpu.add_output_device(6, Box::new(DummyOutputDevice{}));

        Console {
            cpu,
        }
    }
}