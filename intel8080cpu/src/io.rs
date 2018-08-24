use intel8080cpu::Intel8080Cpu;
use super::CpuError;

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn execute_in(&mut self, id: u8) -> Result<(), CpuError> {
        let val = match self.inputs.get_mut(id as usize) {
            Some(Some(device)) => Ok(device.read()),
            _ => Err(CpuError::InputDeviceNotConfigured { id }),
        }?;
        self.save_to_a(val)
    }

    pub(crate) fn execute_out(&mut self, id: u8) -> Result<(), CpuError> {
        let a_value = self.get_current_a_value()?;
        match self.outputs.get_mut(id as usize) {
            Some(Some(device)) => {
                device.write(a_value);
                Ok(())
            },
            _ => Err(CpuError::OutputDeviceNotConfigured { id }),
        }
    }
}

#[cfg(test)]
mod tests {
    use intel8080cpu::{Intel8080Cpu, ROM_MEMORY_LIMIT};
    use instruction::Intel8080Instruction;
    use std::boxed::Box;
    use super::super::cpu::{Cpu, InputDevice, OutputDevice, WithPorts};

    #[test]
    fn it_should_execute_in() {
        struct TestInputDevice;
        impl InputDevice for TestInputDevice {
            fn read(&mut self) -> u8 {
                42
            }
        }

        let input_device = TestInputDevice {};
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.add_input_device(0, Box::new(input_device));
        cpu.execute_instruction(Intel8080Instruction::In { byte: 0 }).unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 42);
    }

    #[test]
    fn it_should_execute_out() {
        struct TestOutputDevice { }
        impl OutputDevice for TestOutputDevice {
            fn write(&mut self, new_value: u8) {
                assert_eq!(new_value, 42);
            }
        }
        let output_device = TestOutputDevice { };
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.add_output_device(0, Box::new(output_device));
        cpu.save_to_a(42).unwrap();
        cpu.execute_instruction(Intel8080Instruction::Out { byte: 0 }).unwrap();
    }
}