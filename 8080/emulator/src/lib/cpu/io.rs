use cpu::cpu::Cpu;

impl<'a> Cpu<'a> {
    pub(crate) fn execute_in(&mut self, id: u8) {
        let new_a = match self.inputs.get_mut(id as usize) {
            Some(device) => {
                device.read()
            },
            _ => panic!("Input device {} not configured", id),
        };
        self.save_to_a(new_a);
    }

    pub(crate) fn execute_out(&mut self, id: u8) {
        let a_value = self.get_current_a_value();
        match self.outputs.get_mut(id as usize) {
            Some(device) => device.write(a_value),
            _ => panic!("Output device {} not configured", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, InputDevice, OutputDevice, ROM_MEMORY_LIMIT};
    use disassembler_8080::Instruction;

    #[test]
    fn it_should_execute_in() {
        struct TestInputDevice;
        impl InputDevice for TestInputDevice {
            fn read(&mut self) -> u8 {
                42
            }
        }

        let mut input_device = TestInputDevice {};
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.add_input_device(&mut input_device);
        cpu.execute_instruction(Instruction::In { byte: 0 });
        assert_eq!(cpu.get_current_a_value(), 42);
    }

    #[test]
    fn it_should_execute_out() {
        struct TestOutputDevice { value: u8 }
        impl OutputDevice for TestOutputDevice {
            fn write(&mut self, new_value: u8) {
                self.value = new_value;
            }
        }

        let mut output_device = TestOutputDevice { value: 0 };
        {
            let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
            cpu.add_output_device(&mut output_device);
            cpu.save_to_a(42);
            cpu.execute_instruction(Instruction::Out { byte: 0 });
        }
        assert_eq!(output_device.value, 42);
    }
}