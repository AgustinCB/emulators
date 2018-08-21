use intel8080cpu::{Intel8080Cpu, Location, State};
use instruction::Intel8080Instruction;
use std::cmp::min;
use super::CpuError;

impl<'a> Intel8080Cpu<'a> {
    pub fn execute(&mut self) -> Result<u8, CpuError> {
        let instruction = Intel8080Instruction::from_bytes(self.get_next_instruction_bytes());
        if !self.can_run(&instruction) {
            return Ok(0);
        }
        println!("{}", instruction.to_string());
        self.pc += instruction.size() as u16;
        let cycles = self.get_cycles_for_instruction(&instruction);
        self.execute_instruction(instruction)?;
        Ok(cycles)
    }

    pub fn execute_instruction(&mut self, instruction: Intel8080Instruction) -> Result<(), CpuError> {
        if !self.can_run(&instruction) {
            return Ok(());
        }
        match instruction {
            Intel8080Instruction::Adc { source: Location::Register { register } } =>
                self.execute_adc_by_register(&register)?,
            Intel8080Instruction::Adc { source: Location::Memory } => self.execute_adc_by_memory()?,
            Intel8080Instruction::Add { source: Location::Register { register } } =>
                self.execute_add_by_register(&register)?,
            Intel8080Instruction::Add { source: Location::Memory } => self.execute_add_by_memory()?,
            Intel8080Instruction::Aci { byte } => self.execute_aci(byte)?,
            Intel8080Instruction::Adi { byte } => self.execute_adi(byte)?,
            Intel8080Instruction::Ana { source: Location::Register { register } } =>
                self.execute_ana_by_register(&register)?,
            Intel8080Instruction::Ana { source: Location::Memory } => self.execute_ana_by_memory()?,
            Intel8080Instruction::Ani { byte } => self.execute_ani(byte)?,
            Intel8080Instruction::Call { address } =>
                self.execute_call(address[1], address[0])?,
            Intel8080Instruction::Cc { address } =>
                self.execute_cc(address[1], address[0]),
            Intel8080Instruction::Cm { address } =>
                self.execute_cm(address[1], address[0]),
            Intel8080Instruction::Cma => self.execute_cma()?,
            Intel8080Instruction::Cmc => self.execute_cmc(),
            Intel8080Instruction::Cmp { source: Location::Register { register } } =>
                self.execute_cmp_by_register(&register)?,
            Intel8080Instruction::Cmp { source: Location::Memory } => self.execute_cmp_by_memory()?,
            Intel8080Instruction::Cnc { address } =>
                self.execute_cnc(address[1], address[0]),
            Intel8080Instruction::Cnz { address } =>
                self.execute_cnz(address[1], address[0]),
            Intel8080Instruction::Cp { address } =>
                self.execute_cp(address[1], address[0]),
            Intel8080Instruction::Cpe { address } =>
                self.execute_cpe(address[1], address[0]),
            Intel8080Instruction::Cpo { address } =>
                self.execute_cpo(address[1], address[0]),
            Intel8080Instruction::Cpi { byte } => self.execute_cpi(byte)?,
            Intel8080Instruction::Cz { address } =>
                self.execute_cz(address[1], address[0]),
            Intel8080Instruction::Daa => self.execute_daa()?,
            Intel8080Instruction::Dad { register } => self.execute_dad(&register)?,
            Intel8080Instruction::Dcr { source: Location::Register { register } } =>
                self.execute_dcr_by_register(&register)?,
            Intel8080Instruction::Dcr { source: Location::Memory } => self.execute_dcr_by_memory(),
            Intel8080Instruction::Dcx { register } => self.execute_dcx(&register)?,
            Intel8080Instruction::Di => self.execute_di(),
            Intel8080Instruction::Ei => self.execute_ei(),
            Intel8080Instruction::Hlt => self.execute_hlt(),
            Intel8080Instruction::In { byte } => self.execute_in(byte)?,
            Intel8080Instruction::Inr { source: Location::Register { register } } =>
                self.execute_inr_by_register(&register)?,
            Intel8080Instruction::Inr { source: Location::Memory } => self.execute_inr_by_memory(),
            Intel8080Instruction::Inx { register } => self.execute_inx(&register)?,
            Intel8080Instruction::Jc { address } =>
                self.execute_jc(address[1], address[0]),
            Intel8080Instruction::Jm { address } =>
                self.execute_jm(address[1], address[0]),
            Intel8080Instruction::Jnc { address } =>
                self.execute_jnc(address[1], address[0]),
            Intel8080Instruction::Jnz { address } =>
                self.execute_jnz(address[1], address[0]),
            Intel8080Instruction::Jmp { address } =>
                self.execute_jmp(address[1], address[0]),
            Intel8080Instruction::Jp { address } =>
                self.execute_jp(address[1], address[0]),
            Intel8080Instruction::Jpe { address } =>
                self.execute_jpe(address[1], address[0]),
            Intel8080Instruction::Jpo { address } =>
                self.execute_jpo(address[1], address[0]),
            Intel8080Instruction::Jz { address } =>
                self.execute_jz(address[1], address[0]),
            Intel8080Instruction::Lda { address } =>
                self.execute_lda(address[1], address[0])?,
            Intel8080Instruction::Ldax { register } => self.execute_ldax(&register)?,
            Intel8080Instruction::Lhld { address } =>
                self.execute_lhld(address[1], address[0])?,
            Intel8080Instruction::Lxi { register, low_byte, high_byte } =>
                self.execute_lxi(&register, high_byte, low_byte)?,
            Intel8080Instruction::Mov { destiny, source } => self.execute_mov(&destiny, &source)?,
            Intel8080Instruction::Mvi { source: Location::Memory, byte} => self.execute_mvi_to_memory(byte),
            Intel8080Instruction::Mvi { source: Location::Register { register }, byte } =>
                self.save_to_single_register(byte, &register)?,
            Intel8080Instruction::Noop => self.execute_noop(),
            Intel8080Instruction::Pchl => self.execute_pchl(),
            Intel8080Instruction::Pop { register } => self.execute_pop(&register)?,
            Intel8080Instruction::Push { register } => self.execute_push(&register)?,
            Intel8080Instruction::Ora { source: Location::Register { register } } =>
                self.execute_ora_by_register(&register)?,
            Intel8080Instruction::Ora { source: Location::Memory } => self.execute_ora_by_memory()?,
            Intel8080Instruction::Ori { byte } => self.execute_ori(byte)?,
            Intel8080Instruction::Out { byte } => self.execute_out(byte)?,
            Intel8080Instruction::Ral => self.execute_ral()?,
            Intel8080Instruction::Rar => self.execute_rar()?,
            Intel8080Instruction::Rc => self.execute_rc(),
            Intel8080Instruction::Ret => self.execute_ret(),
            Intel8080Instruction::Rlc => self.execute_rlc()?,
            Intel8080Instruction::Rm => self.execute_rm(),
            Intel8080Instruction::Rnc => self.execute_rnc(),
            Intel8080Instruction::Rnz => self.execute_rnz(),
            Intel8080Instruction::Rp => self.execute_rp(),
            Intel8080Instruction::Rpe => self.execute_rpe(),
            Intel8080Instruction::Rpo => self.execute_rpo(),
            Intel8080Instruction::Rrc => self.execute_rrc()?,
            Intel8080Instruction::Rst { value } => self.execute_rst(value),
            Intel8080Instruction::Rz => self.execute_rz(),
            Intel8080Instruction::Sbb { source: Location::Register { register } } =>
                self.execute_sbb_by_register(&register)?,
            Intel8080Instruction::Sbb { source: Location::Memory } => self.execute_sbb_by_memory()?,
            Intel8080Instruction::Sbi { byte } => self.execute_sbi(byte)?,
            Intel8080Instruction::Shld { address } =>
                self.execute_shld(address[1], address[0])?,
            Intel8080Instruction::Sta { address } =>
                self.execute_sta(address[1], address[0])?,
            Intel8080Instruction::Stax { register } => self.execute_stax(&register)?,
            Intel8080Instruction::Stc => self.execute_stc(),
            Intel8080Instruction::Sphl => self.execute_sphl(),
            Intel8080Instruction::Sub { source: Location::Register { register } } =>
                self.execute_sub_by_register(&register)?,
            Intel8080Instruction::Sub { source: Location::Memory } => self.execute_sub_by_memory()?,
            Intel8080Instruction::Sui { byte } => self.execute_sui(byte)?,
            Intel8080Instruction::Xchg => self.execute_xchg()?,
            Intel8080Instruction::Xra { source: Location::Register { register } } =>
                self.execute_xra_by_register(&register)?,
            Intel8080Instruction::Xra { source: Location::Memory } => self.execute_xra_by_memory()?,
            Intel8080Instruction::Xri { byte } => self.execute_xri(byte)?,
            Intel8080Instruction::Xthl => self.execute_xthl()?,
        };
        Ok(())
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.pc as usize;
        let to = min(from+3, self.memory.len());
        &self.memory[from..to]
    }

    #[inline]
    fn can_run(&self, instruction: &Intel8080Instruction) -> bool {
        match instruction {
            Intel8080Instruction::Rst { value: _ } => true,
            _ if self.state == State::Running => true,
            _ => false,
        }
    }

    fn execute_noop(&self) {}
}

#[cfg(test)]
mod tests {
    use intel8080cpu::{Intel8080Cpu, ROM_MEMORY_LIMIT, State};

    #[test]
    fn it_should_execute_instruction_when_running() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.state = State::Running;
        cpu.pc = 0;
        cpu.memory[0] = 0x00;
        cpu.execute().unwrap();
        assert_eq!(cpu.pc, 0x01);
    }

    #[test]
    fn it_shouldnt_execute_instruction_when_stopped() {
        let mut cpu = Intel8080Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.state = State::Stopped;
        cpu.pc = 0;
        cpu.memory[0] = 0x00;
        cpu.execute().unwrap();
        assert_eq!(cpu.pc, 0x00);
    }
}