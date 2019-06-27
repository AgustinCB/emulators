use super::CpuError;
use helpers::two_bytes_to_word;
use intel8080cpu::{Intel8080Cpu, Location, RegisterType};

impl<'a> Intel8080Cpu<'a> {
    pub(crate) fn execute_lda(&mut self, high_byte: u8, low_byte: u8) -> Result<(), CpuError> {
        let source_address = two_bytes_to_word(high_byte, low_byte) as usize;
        let value = self.memory[source_address];
        self.save_to_a(value)
    }

    pub(crate) fn execute_ldax(&mut self, register: RegisterType) -> Result<(), CpuError> {
        let source_address = match register {
            RegisterType::B => self.get_current_bc_value(),
            RegisterType::D => self.get_current_de_value(),
            _ => panic!(
                "Register {} is not a valid input of LDAX",
                register.to_string()
            ),
        } as usize;
        let value = self.memory[source_address];
        self.save_to_a(value)
    }

    pub(crate) fn execute_lhld(&mut self, high_byte: u8, low_byte: u8) -> Result<(), CpuError> {
        let destiny_address = two_bytes_to_word(high_byte, low_byte) as usize;
        let l_value = self.memory[destiny_address];
        let h_value = self.memory[destiny_address + 1];
        self.save_to_single_register(h_value, RegisterType::H)?;
        self.save_to_single_register(l_value, RegisterType::L)
    }

    pub(crate) fn execute_lxi(
        &mut self,
        register_type: RegisterType,
        high_byte: u8,
        low_byte: u8,
    ) -> Result<(), CpuError> {
        match register_type {
            RegisterType::B => {
                self.save_to_single_register(high_byte, RegisterType::B)?;
                self.save_to_single_register(low_byte, RegisterType::C)
            }
            RegisterType::D => {
                self.save_to_single_register(high_byte, RegisterType::D)?;
                self.save_to_single_register(low_byte, RegisterType::E)
            }
            RegisterType::H => {
                self.save_to_single_register(high_byte, RegisterType::H)?;
                self.save_to_single_register(low_byte, RegisterType::L)
            }
            RegisterType::Sp => {
                self.save_to_sp(two_bytes_to_word(high_byte, low_byte));
                Ok(())
            }
            _ => Err(CpuError::InvalidRegisterArgument {
                register: register_type,
            }),
        }
    }

    #[inline]
    pub(crate) fn execute_mov(
        &mut self,
        destiny: Location,
        source: Location,
    ) -> Result<(), CpuError> {
        match (destiny, source) {
            (Location::Register { register: destiny }, Location::Register { register: source }) => {
                self.execute_mov_register_to_register(destiny, source)
            }
            (Location::Register { register: destiny }, Location::Memory) => {
                self.execute_mov_memory_to_register(destiny)
            }
            (Location::Memory, Location::Register { register: source }) => {
                self.execute_mov_register_to_memory(source)
            }
            (Location::Memory, Location::Memory) => Err(CpuError::InvalidMemoryAccess),
        }
    }

    #[inline]
    pub(crate) fn execute_mvi_to_memory(&mut self, byte: u8) {
        let address = self.get_current_hl_value();
        self.memory[address as usize] = byte;
    }

    pub(crate) fn execute_shld(&mut self, high_byte: u8, low_byte: u8) -> Result<(), CpuError> {
        let h_value = self.get_current_single_register_value(RegisterType::H)?;
        let l_value = self.get_current_single_register_value(RegisterType::L)?;
        let destiny_address = two_bytes_to_word(high_byte, low_byte) as usize;
        self.memory[destiny_address] = l_value;
        self.memory[destiny_address + 1] = h_value;
        Ok(())
    }

    pub(crate) fn execute_sphl(&mut self) {
        let hl = self.get_current_hl_value();
        self.save_to_sp(hl);
    }

    pub(crate) fn execute_sta(&mut self, high_byte: u8, low_byte: u8) -> Result<(), CpuError> {
        let value = self.get_current_a_value()?;
        let destiny_address = two_bytes_to_word(high_byte, low_byte);
        self.memory[destiny_address as usize] = value;
        Ok(())
    }

    pub(crate) fn execute_stax(&mut self, register: RegisterType) -> Result<(), CpuError> {
        let value = self.get_current_a_value()?;
        let destiny_address = match register {
            RegisterType::B => self.get_current_bc_value(),
            RegisterType::D => self.get_current_de_value(),
            _ => panic!(
                "Register {} is not a valid input of STAX",
                register.to_string()
            ),
        } as usize;
        self.memory[destiny_address] = value;
        Ok(())
    }

    pub(crate) fn execute_xchg(&mut self) -> Result<(), CpuError> {
        let d_value = self.get_current_single_register_value(RegisterType::D)?;
        let e_value = self.get_current_single_register_value(RegisterType::E)?;
        let h_value = self.get_current_single_register_value(RegisterType::H)?;
        let l_value = self.get_current_single_register_value(RegisterType::L)?;
        self.save_to_single_register(h_value, RegisterType::D)?;
        self.save_to_single_register(l_value, RegisterType::E)?;
        self.save_to_single_register(d_value, RegisterType::H)?;
        self.save_to_single_register(e_value, RegisterType::L)
    }

    pub(crate) fn execute_xthl(&mut self) -> Result<(), CpuError> {
        let sp = self.get_current_sp_value() as usize;
        let first_byte = self.memory[sp + 1];
        let second_byte = self.memory[sp];
        let h_value = self.get_current_single_register_value(RegisterType::H)?;
        let l_value = self.get_current_single_register_value(RegisterType::L)?;
        self.memory[sp + 1] = h_value;
        self.memory[sp] = l_value;
        self.save_to_single_register(first_byte, RegisterType::H)?;
        self.save_to_single_register(second_byte, RegisterType::L)
    }

    #[inline]
    fn execute_mov_register_to_register(
        &mut self,
        destiny: RegisterType,
        source: RegisterType,
    ) -> Result<(), CpuError> {
        let source_value = self.get_current_single_register_value(source)?;
        self.save_to_single_register(source_value, destiny)
    }

    #[inline]
    fn execute_mov_memory_to_register(&mut self, destiny: RegisterType) -> Result<(), CpuError> {
        let source_value = self.get_value_in_memory_at_hl();
        self.save_to_single_register(source_value, destiny)
    }

    #[inline]
    fn execute_mov_register_to_memory(&mut self, source: RegisterType) -> Result<(), CpuError> {
        let source_value = self.get_current_single_register_value(source)?;
        self.set_value_in_memory_at_hl(source_value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::cpu::Cpu;
    use instruction::Intel8080Instruction;
    use intel8080cpu::{Intel8080Cpu, Location, RegisterType, ROM_MEMORY_LIMIT};

    fn get_ldax_ready_cpu<'a>(register: RegisterType) -> Intel8080Cpu<'a> {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.memory[0x938b] = 42;

        match register {
            RegisterType::B => {
                cpu.save_to_single_register(0x93, RegisterType::B).unwrap();
                cpu.save_to_single_register(0x8b, RegisterType::C).unwrap();
            }
            RegisterType::D => {
                cpu.save_to_single_register(0x93, RegisterType::D).unwrap();
                cpu.save_to_single_register(0x8b, RegisterType::E).unwrap();
            }
            _ => panic!(
                "Register {} is not a valid argument to ldax.",
                register.to_string()
            ),
        };
        cpu
    }

    fn get_stax_ready_cpu<'a>(register: RegisterType) -> Intel8080Cpu<'a> {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42).unwrap();
        match register {
            RegisterType::B => {
                cpu.save_to_single_register(0x3f, RegisterType::B).unwrap();
                cpu.save_to_single_register(0x16, RegisterType::C).unwrap();
            }
            RegisterType::D => {
                cpu.save_to_single_register(0x3f, RegisterType::D).unwrap();
                cpu.save_to_single_register(0x16, RegisterType::E).unwrap();
            }
            _ => panic!(
                "Register {} is not a valid argument to stax.",
                register.to_string()
            ),
        };
        cpu
    }

    #[test]
    fn it_should_execute_lda() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42).unwrap();
        cpu.memory[0x24] = 0x24;
        cpu.execute_instruction(&Intel8080Instruction::Lda {
            address: [0x24, 0x00],
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 0x24);
    }

    #[test]
    fn it_should_execute_ldax_from_b() {
        let mut cpu = get_ldax_ready_cpu(RegisterType::B);
        cpu.execute_instruction(&Intel8080Instruction::Ldax {
            register: RegisterType::B,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 42);
    }

    #[test]
    fn it_should_execute_ldax_from_d() {
        let mut cpu = get_ldax_ready_cpu(RegisterType::D);
        cpu.execute_instruction(&Intel8080Instruction::Ldax {
            register: RegisterType::D,
        })
        .unwrap();
        assert_eq!(cpu.get_current_a_value().unwrap(), 42);
    }

    #[test]
    fn it_should_execute_lhld() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.memory[0x025b] = 0xff;
        cpu.memory[0x025c] = 0x03;
        cpu.execute_instruction(&Intel8080Instruction::Lhld {
            address: [0x5b, 0x02],
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::H)
                .unwrap(),
            0x03
        );
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::L)
                .unwrap(),
            0xff
        );
    }

    #[test]
    fn it_should_execute_lxi_to_b() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_instruction(&Intel8080Instruction::Lxi {
            register: RegisterType::B,
            high_byte: 0x42,
            low_byte: 0x24,
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::B)
                .unwrap(),
            0x42
        );
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::C)
                .unwrap(),
            0x24
        );
    }

    #[test]
    fn it_should_execute_lxi_to_d() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_instruction(&Intel8080Instruction::Lxi {
            register: RegisterType::D,
            high_byte: 0x42,
            low_byte: 0x24,
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::D)
                .unwrap(),
            0x42
        );
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::E)
                .unwrap(),
            0x24
        );
    }

    #[test]
    fn it_should_execute_lxi_to_h() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_instruction(&Intel8080Instruction::Lxi {
            register: RegisterType::H,
            high_byte: 0x42,
            low_byte: 0x24,
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::H)
                .unwrap(),
            0x42
        );
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::L)
                .unwrap(),
            0x24
        );
    }

    #[test]
    fn it_should_execute_lxi_to_sp() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.execute_instruction(&Intel8080Instruction::Lxi {
            register: RegisterType::Sp,
            high_byte: 0x42,
            low_byte: 0x24,
        })
        .unwrap();
        assert_eq!(cpu.get_current_sp_value(), 0x4224);
    }

    #[test]
    fn it_should_execute_mov_from_register_to_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x42, RegisterType::B).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Mov {
            destiny: Location::Register {
                register: RegisterType::C,
            },
            source: Location::Register {
                register: RegisterType::B,
            },
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::C)
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_execute_mov_from_memory_to_register() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.memory[0x42] = 0x24;
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x42, RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Mov {
            destiny: Location::Register {
                register: RegisterType::C,
            },
            source: Location::Memory,
        })
        .unwrap();
        assert_eq!(
            cpu.get_current_single_register_value(RegisterType::C)
                .unwrap(),
            0x24
        );
    }

    #[test]
    fn it_should_execute_mov_from_register_to_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x24, RegisterType::C).unwrap();
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x42, RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Mov {
            destiny: Location::Memory,
            source: Location::Register {
                register: RegisterType::C,
            },
        })
        .unwrap();
        assert_eq!(cpu.memory[0x42], 0x24);
    }

    #[test]
    fn it_should_execute_mvi_to_memory() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x42, RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Mvi {
            source: Location::Memory,
            byte: 0x24,
        })
        .unwrap();
        assert_eq!(cpu.memory[0x42], 0x24);
    }

    #[test]
    fn it_should_execute_shld() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0xae, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x29, RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Shld {
            address: [0x0a, 0x01],
        })
        .unwrap();
        assert_eq!(cpu.memory[0x010a], 0x29);
        assert_eq!(cpu.memory[0x010b], 0xae);
    }

    #[test]
    fn it_should_execut_sphl() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x00, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x42, RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Sphl)
            .unwrap();
        assert_eq!(cpu.get_current_sp_value(), 0x42);
        assert_eq!(cpu.get_current_hl_value(), 0x42);
    }

    #[test]
    fn it_should_execute_sta() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_a(0x42).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Sta {
            address: [0x24, 0x00],
        })
        .unwrap();
        assert_eq!(cpu.memory[0x24], 0x42);
    }

    #[test]
    fn it_should_execute_stax_for_b() {
        let mut cpu = get_stax_ready_cpu(RegisterType::B);
        cpu.execute_instruction(&Intel8080Instruction::Stax {
            register: RegisterType::B,
        })
        .unwrap();
        assert_eq!(cpu.memory[0x3f16], 0x42);
    }

    #[test]
    fn it_should_execute_stax_for_d() {
        let mut cpu = get_stax_ready_cpu(RegisterType::D);
        cpu.execute_instruction(&Intel8080Instruction::Stax {
            register: RegisterType::D,
        })
        .unwrap();
        assert_eq!(cpu.memory[0x3f16], 0x42);
    }

    #[test]
    fn it_should_execute_xchg() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_single_register(0x42, RegisterType::D).unwrap();
        cpu.save_to_single_register(0x24, RegisterType::E).unwrap();
        cpu.save_to_single_register(0x24, RegisterType::H).unwrap();
        cpu.save_to_single_register(0x42, RegisterType::L).unwrap();
        cpu.execute_instruction(&Intel8080Instruction::Xchg)
            .unwrap();
        assert_eq!(cpu.get_current_de_value(), 0x2442);
        assert_eq!(cpu.get_current_hl_value(), 0x4224);
    }

    #[test]
    fn it_should_execute_xthl() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_sp(0);
        cpu.memory[0] = 0x42;
        cpu.memory[1] = 0x24;
        cpu.execute_instruction(&Intel8080Instruction::Xthl)
            .unwrap();
        assert_eq!(cpu.get_current_hl_value(), 0x2442);
    }
}
