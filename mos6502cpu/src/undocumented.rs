use bit_utils::{two_complement, word_to_two_bytes};
use instruction::AddressingMode;
use mos6502cpu::ProcessorStatus;
use {CpuError, CpuResult, Mos6502Cpu};

// Implementation based on http://www.oxyron.de/html/opcodes02.html
impl Mos6502Cpu {
    pub(crate) fn execute_ahx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::IndirectIndexed { .. } | AddressingMode::AbsoluteIndexedY { .. } => {
                let address = self.get_address_from_addressing_mode(addressing_mode)?;
                let (_, high_byte) = word_to_two_bytes(address);
                let answer = self.registers.x & self.registers.a & high_byte;
                self.set_value_to_addressing_mode(addressing_mode, answer)
            }
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn execute_alr(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { .. } = addressing_mode {
            self.execute_and_unchecked(addressing_mode)?;
            self.execute_lsr_unchecked(addressing_mode)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_anc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { byte } = addressing_mode {
            let future_carry = byte & 0x80 > 0;
            self.execute_and_unchecked(addressing_mode)?;
            self.registers.p.carry = future_carry;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_arr(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { .. } = addressing_mode {
            self.execute_and_unchecked(addressing_mode)?;
            self.execute_ror_unchecked(addressing_mode)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_axs(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { byte } = addressing_mode {
            let operand1 = self.registers.x & self.registers.a;
            let operand2 = two_complement(*byte);
            self.registers.x = self.compare(operand1, operand2) as u8;
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_dcp(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        self.execute_dec_unchecked(addressing_mode)?;
        self.execute_cmp_unchecked(addressing_mode)
    }

    pub(crate) fn execute_isc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        self.execute_inc_unchecked(addressing_mode)?;
        self.execute_sbc_unchecked(addressing_mode)
    }

    pub(crate) fn execute_las(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::AbsoluteIndexedY { .. } = addressing_mode {
            let value = self.get_value_from_addressing_mode(addressing_mode)?;
            let answer = value & self.registers.p.to_byte();
            self.registers.x = answer;
            self.registers.a = answer;
            self.registers.p = ProcessorStatus::from_byte(answer);
            Ok(())
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_lax(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { .. } => {
                self.execute_lda_unchecked(addressing_mode)?;
                self.execute_tax_unchecked();
                Ok(())
            }
            AddressingMode::ZeroPage { .. }
            | AddressingMode::ZeroPageIndexedY { .. }
            | AddressingMode::IndexedIndirect { .. }
            | AddressingMode::IndirectIndexed { .. }
            | AddressingMode::Absolute { .. }
            | AddressingMode::AbsoluteIndexedY { .. } => {
                self.execute_lda_unchecked(addressing_mode)?;
                self.execute_ldx_unchecked(addressing_mode)
            }
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn execute_rla(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        self.execute_rol_unchecked(addressing_mode)?;
        self.execute_and_unchecked(addressing_mode)
    }

    pub(crate) fn execute_rra(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        self.execute_ror_unchecked(addressing_mode)?;
        self.execute_adc_unchecked(addressing_mode)
    }

    pub(crate) fn execute_sax(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::IndexedIndirect { .. }
            | AddressingMode::ZeroPage { .. }
            | AddressingMode::ZeroPageIndexedY { .. }
            | AddressingMode::Absolute { .. } => {
                let answer = self.registers.a & self.registers.x;
                self.set_value_to_addressing_mode(addressing_mode, answer)
            }
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn execute_shx(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::AbsoluteIndexedY { .. } = addressing_mode {
            let address = self.get_address_from_addressing_mode(addressing_mode)?;
            let (_, high_byte) = word_to_two_bytes(address);
            let answer = self.registers.x & high_byte;
            self.set_value_to_addressing_mode(addressing_mode, answer)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_shy(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::AbsoluteIndexedX { .. } = addressing_mode {
            let address = self.get_address_from_addressing_mode(addressing_mode)?;
            let (_, high_byte) = word_to_two_bytes(address);
            let answer = self.registers.y & high_byte;
            self.set_value_to_addressing_mode(addressing_mode, answer)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_slo(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        self.execute_asl_unchecked(addressing_mode)?;
        self.execute_ora_unchecked(addressing_mode)
    }

    pub(crate) fn execute_sre(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.check_data_store_address(addressing_mode)?;
        self.execute_lsr_unchecked(addressing_mode)?;
        self.execute_eor_unchecked(addressing_mode)
    }

    pub(crate) fn execute_tas(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::AbsoluteIndexedY { .. } = addressing_mode {
            self.registers.p = ProcessorStatus::from_byte(self.registers.a & self.registers.x);
            self.execute_ahx(addressing_mode)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }

    pub(crate) fn execute_xaa(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { .. } = addressing_mode {
            self.execute_txa_unchecked();
            self.execute_and_unchecked(addressing_mode)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }
}
