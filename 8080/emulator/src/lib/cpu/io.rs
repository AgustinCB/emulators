use cpu::cpu::Cpu;

impl<'a> Cpu<'a> {
    pub(crate) fn execute_in(&mut self, id: u8) {
        let new_a = match self.inputs.get_mut(id as usize) {
            Some(Some(device)) => {
                device.read(id)
            },
            _ => panic!("Input device {} not configured", id),
        };
        self.save_to_a(new_a);
    }

    pub(crate) fn execute_out(&mut self, id: u8) {
        let a_value = self.get_current_a_value();
        match self.outputs.get_mut(id as usize) {
            Some(Some(device)) => device.write(id, a_value),
            _ => panic!("Output device {} not configured", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, InputDevice, OutputDevice, ROM_MEMORY_LIMIT};
    use cpu::instruction::Instruction;
    use std::boxed::Box;

    #[test]
    fn it_should_execute_in() {
        struct TestInputDevice;
        impl InputDevice for TestInputDevice {
            fn read(&mut self, _: u8) -> u8 {
                42
            }
        }

        let input_device = TestInputDevice {};
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.add_input_device(0, Box::new(input_device));
        cpu.execute_instruction(Instruction::In { byte: 0 });
        assert_eq!(cpu.get_current_a_value(), 42);
    }

    #[test]
    fn it_should_execute_out() {
        struct TestOutputDevice { }
        impl OutputDevice for TestOutputDevice {
            fn write(&mut self, _: u8, new_value: u8) {
                assert_eq!(new_value, 42);
            }
        }
        let output_device = TestOutputDevice { };
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.add_output_device(0, Box::new(output_device));
        cpu.save_to_a(42);
        cpu.execute_instruction(Instruction::Out { byte: 0 });
    }
}