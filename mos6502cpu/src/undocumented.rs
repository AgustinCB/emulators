use {Mos6502Cpu, CpuResult};
use instruction::AddressingMode;

// Implementation based on http://www.oxyron.de/html/opcodes02.html
impl Mos6502Cpu {
    pub(crate) fn execute_slo(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.execute_asl(addressing_mode)?;
        self.execute_ora(addressing_mode)?;
        Ok(())
    }

    pub(crate) fn execute_rla(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.execute_rol(addressing_mode)?;
        self.execute_and(addressing_mode)?;
        Ok(())
    }

    pub(crate) fn execute_sre(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.execute_lsr(addressing_mode)?;
        self.execute_eor(addressing_mode)?;
        Ok(())
    }

    pub(crate) fn execute_rra(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.execute_ror(addressing_mode)?;
        self.execute_adc(addressing_mode)?;
        Ok(())
    }

    pub(crate) fn execute_dcp(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.execute_dec(addressing_mode)?;
        self.execute_cmp(addressing_mode)?;
        Ok(())
    }

    pub(crate) fn execute_isc(&mut self, addressing_mode: &AddressingMode) -> CpuResult {
        self.execute_inc(addressing_mode)?;
        self.execute_sbc(addressing_mode)?;
        Ok(())
    }
}