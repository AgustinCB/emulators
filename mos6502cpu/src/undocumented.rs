use {Mos6502Cpu, CpuError, CpuResult};
use bit_utils::two_complement;
use instruction::AddressingMode;

// Implementation based on http://www.oxyron.de/html/opcodes02.html
impl Mos6502Cpu {
    pub(crate) fn execute_alr(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { byte: _ } = addressing_mode {
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
        if let AddressingMode::Immediate { byte: _ } = addressing_mode {
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

    pub(crate) fn execute_lax(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        match addressing_mode {
            AddressingMode::Immediate { byte: _ } => {
                self.execute_lda_unchecked(addressing_mode)?;
                self.execute_tax_unchecked();
                Ok(())
            },
            AddressingMode::ZeroPage { byte: _ } |
            AddressingMode::ZeroPageIndexedY { byte: _ } |
            AddressingMode::IndexedIndirect { byte: _ } |
            AddressingMode::IndirectIndexed { byte: _ } |
            AddressingMode::Absolute { low_byte: _, high_byte: _ } |
            AddressingMode::AbsoluteIndexedY { low_byte: _, high_byte: _ } => {
                self.execute_lda_unchecked(addressing_mode)?;
                self.execute_ldx_unchecked(addressing_mode)
            },
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
            AddressingMode::IndexedIndirect { byte: _ } |
            AddressingMode::ZeroPage { byte: _ } |
            AddressingMode::ZeroPageIndexedY { byte: _ } |
            AddressingMode::Absolute { low_byte: _, high_byte: _ } => {
                let answer = self.registers.a & self.registers.x;
                self.set_value_to_addressing_mode(addressing_mode, answer)
            },
            _ => Err(CpuError::InvalidAddressingMode),
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

    pub(crate) fn execute_xaa(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        if let AddressingMode::Immediate { byte: _ } = addressing_mode {
            self.execute_txa_unchecked();
            self.execute_and_unchecked(addressing_mode)
        } else {
            Err(CpuError::InvalidAddressingMode)
        }
    }
}