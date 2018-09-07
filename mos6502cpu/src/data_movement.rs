use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

impl Mos6502Cpu {
    pub(crate) fn execute_lda(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_alu_address(addressing_mode)?;
        self.execute_lda_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_lda_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        self.registers.a = value;
        self.update_zero_flag(value);
        self.update_negative_flag(value);
        Ok(())
    }

    pub(crate) fn execute_ldx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_load_address_y(addressing_mode)?;
        self.execute_ldx_unchecked(addressing_mode)
    }

    #[inline]
    pub(crate) fn execute_ldx_unchecked(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        self.registers.x = value;
        self.update_zero_flag(value);
        self.update_negative_flag(value);
        Ok(())
    }

    pub(crate) fn execute_ldy(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_load_address_x(addressing_mode)?;
        let value = self.get_value_from_addressing_mode(addressing_mode)?;
        self.registers.y = value;
        self.update_zero_flag(value);
        self.update_negative_flag(value);
        Ok(())
    }

    pub(crate) fn execute_sta(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        let address = self.get_address_from_addressing_mode(addressing_mode)? as usize;
        self.memory[address] = self.registers.a;
        Ok(())
    }

    pub(crate) fn execute_stx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::ZeroPage { byte: _ } |
            AddressingMode::ZeroPageIndexedY { byte: _ } |
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => {
                let address = self.get_address_from_addressing_mode(addressing_mode)? as usize;
                self.memory[address] = self.registers.x;
                Ok(())
            },
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn execute_sty(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::ZeroPage { byte: _ } |
            AddressingMode::ZeroPageIndexedX { byte: _ } |
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => {
                let address = self.get_address_from_addressing_mode(addressing_mode)? as usize;
                self.memory[address] = self.registers.y;
                Ok(())
            },
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn execute_tax(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.execute_tax_unchecked();
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    pub(crate) fn execute_tax_unchecked(&mut self) {
        let a_value = self.registers.a;
        self.registers.x = a_value;
        self.update_zero_flag(a_value);
        self.update_negative_flag(a_value);
    }

    pub(crate) fn execute_tay(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let a_value = self.registers.a;
            self.registers.y = a_value;
            self.update_zero_flag(a_value);
            self.update_negative_flag(a_value);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_tsx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let s_value = self.registers.s;
            self.registers.x = s_value;
            self.update_zero_flag(s_value);
            self.update_negative_flag(s_value);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_txa(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            self.execute_txa_unchecked();
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    pub(crate) fn execute_txa_unchecked(&mut self) {
        let x_value = self.registers.x;
        self.registers.a = x_value;
        self.update_zero_flag(x_value);
        self.update_negative_flag(x_value);
    }

    pub(crate) fn execute_txs(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let x_value = self.registers.x;
            self.registers.s = x_value;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_tya(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Implicit = addressing_mode {
            let y_value = self.registers.y;
            self.registers.a = y_value;
            self.update_zero_flag(y_value);
            self.update_negative_flag(y_value);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn check_data_load_address_y(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedY { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    fn check_data_load_address_x(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { byte: _ } => Ok(()),
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }

    #[inline]
    pub(crate) fn check_data_store_address(&self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::ZeroPage { byte: _ } => Ok(()),
            AddressingMode::ZeroPageIndexedX { byte: _ } => Ok(()),
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedX { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } => Ok(()),
            AddressingMode::IndexedIndirect { byte: _ } => Ok(()),
            AddressingMode::IndirectIndexed { byte: _ } => Ok(()),
            _ => Err(CpuError::InvalidAddressingMode)
        }
    }
}

#[cfg(test)]
mod tests {
    use cpu::Cpu;
    use instruction::{AddressingMode, Mos6502Instruction, Mos6502InstructionCode};
    use {AVAILABLE_MEMORY, Mos6502Cpu};

    #[test]
    fn it_should_load_into_accumulator_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lda,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_accumulator_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lda,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_accumulator_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Lda,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_x_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldx,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_x_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldx,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_x_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldx,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_y_and_not_set_anything() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldy,
            addressing_mode: AddressingMode::Immediate { byte: 0x42 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_y_and_set_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldy,
            addressing_mode: AddressingMode::Immediate { byte: 0x00 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_load_into_y_and_set_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Ldy,
            addressing_mode: AddressingMode::Immediate { byte: 0x80 },
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_store_the_accumulator() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0;
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sta,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
    }

    #[test]
    fn it_should_store_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0;
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Stx,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
    }

    #[test]
    fn it_should_store_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0] = 0;
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Sty,
            addressing_mode: AddressingMode::Absolute { high_byte: 0, low_byte: 0 },
        }).unwrap();
        assert_eq!(cpu.memory[0], 0x42);
    }

    #[test]
    fn it_should_move_a_to_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tax,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_a_to_x_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.registers.a = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tax,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_a_to_x_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.registers.a = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tax,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_a_to_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0;
        cpu.registers.a = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tay,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_a_to_y_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x42;
        cpu.registers.a = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tay,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_a_to_y_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0;
        cpu.registers.a = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tay,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.y, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_s_to_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.registers.s = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tsx,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_s_to_x_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x42;
        cpu.registers.s = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tsx,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_s_to_x_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0;
        cpu.registers.s = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tsx,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.x, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_x_to_a() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Txa,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_x_to_a_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.x = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Txa,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_x_to_a_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.registers.x = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Txa,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_x_to_s() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0;
        cpu.registers.x = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Txs,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.s, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_x_to_s_without_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0x42;
        cpu.registers.x = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Txs,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.s, 0);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_x_to_s_without_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.s = 0;
        cpu.registers.x = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Txs,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.s, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_y_to_a() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.registers.y = 0x42;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tya,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
        assert!(!cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_y_to_a_setting_zero() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        cpu.registers.y = 0;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tya,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.p.zero);
        assert!(!cpu.registers.p.negative);
    }

    #[test]
    fn it_should_move_y_to_a_setting_negative() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.registers.y = 0x80;
        cpu.execute_instruction(&Mos6502Instruction {
            instruction: Mos6502InstructionCode::Tya,
            addressing_mode: AddressingMode::Implicit,
        }).unwrap();
        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero);
        assert!(cpu.registers.p.negative);
    }
}