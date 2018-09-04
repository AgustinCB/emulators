use {Mos6502Cpu, CpuError, CpuResult};
use instruction::AddressingMode;

// Implementation based on http://www.oxyron.de/html/opcodes02.html
impl Mos6502Cpu {
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
}