use super::cpu::{Cpu, InputDevice, OutputDevice, ROM_MEMORY_LIMIT};

const NUM_INPUT_DEVICES: usize = 4;
const NUM_OUTPUT_DEVICES: usize = 5;

pub struct Console<'a> {
    cpu: Cpu<'a>,
}

impl<'a> Console<'a> {
    fn new<'b>(
        memory: [u8; ROM_MEMORY_LIMIT],
        input_devices: &'b mut [&'b mut InputDevice; NUM_INPUT_DEVICES],
        output_devices: &'b mut [&'b mut OutputDevice; NUM_OUTPUT_DEVICES]) -> Console<'b> {
        let mut cpu = Cpu::new(memory);
        for d in input_devices.iter_mut() {
            cpu.add_input_device(*d);
        }
        for d in output_devices.iter_mut() {
            cpu.add_output_device(*d);
        }
        Console {
            cpu
        }
    }
}