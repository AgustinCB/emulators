use cpu::cpu::{Cpu, Location, State};
use cpu::instruction::Instruction;
use std::cmp::min;

impl<'a> Cpu<'a> {
    pub fn execute(&mut self) {
        let instruction = Instruction::from_bytes(self.get_next_instruction_bytes());
        if !self.can_run(&instruction) {
            return;
        }
        self.pc += instruction.size() as u16;
        self.execute_instruction(instruction);
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) {
        if !self.can_run(&instruction) {
            return;
        }
        match instruction {
            Instruction::Adc { source: Location::Register { register } } =>
                self.execute_adc_by_register(&register),
            Instruction::Adc { source: Location::Memory } => self.execute_adc_by_memory(),
            Instruction::Add { source: Location::Register { register } } =>
                self.execute_add_by_register(&register),
            Instruction::Add { source: Location::Memory } => self.execute_add_by_memory(),
            Instruction::Aci { byte } => self.execute_aci(byte),
            Instruction::Adi { byte } => self.execute_adi(byte),
            Instruction::Ana { source: Location::Register { register } } =>
                self.execute_ana_by_register(&register),
            Instruction::Ana { source: Location::Memory } => self.execute_ana_by_memory(),
            Instruction::Ani { byte } => self.execute_ani(byte),
            Instruction::Call { address } =>
                self.execute_call(address[1], address[0]),
            Instruction::Cc { address } =>
                self.execute_cc(address[1], address[0]),
            Instruction::Cm { address } =>
                self.execute_cm(address[1], address[0]),
            Instruction::Cma => self.execute_cma(),
            Instruction::Cmc => self.execute_cmc(),
            Instruction::Cmp { source: Location::Register { register } } =>
                self.execute_cmp_by_register(&register),
            Instruction::Cmp { source: Location::Memory } => self.execute_cmp_by_memory(),
            Instruction::Cnc { address } =>
                self.execute_cnc(address[1], address[0]),
            Instruction::Cnz { address } =>
                self.execute_cnz(address[1], address[0]),
            Instruction::Cp { address } =>
                self.execute_cp(address[1], address[0]),
            Instruction::Cpe { address } =>
                self.execute_cpe(address[1], address[0]),
            Instruction::Cpo { address } =>
                self.execute_cpo(address[1], address[0]),
            Instruction::Cpi { byte } => self.execute_cpi(byte),
            Instruction::Cz { address } =>
                self.execute_cz(address[1], address[0]),
            Instruction::Daa => self.execute_daa(),
            Instruction::Dad { register } => self.execute_dad(&register),
            Instruction::Dcr { source: Location::Register { register } } =>
                self.execute_dcr_by_register(&register),
            Instruction::Dcr { source: Location::Memory } => self.execute_dcr_by_memory(),
            Instruction::Dcx { register } => self.execute_dcx(&register),
            Instruction::Di => self.execute_di(),
            Instruction::Ei => self.execute_ei(),
            Instruction::Hlt => self.execute_hlt(),
            Instruction::In { byte } => self.execute_in(byte),
            Instruction::Inr { source: Location::Register { register } } =>
                self.execute_inr_by_register(&register),
            Instruction::Inr { source: Location::Memory } => self.execute_inr_by_memory(),
            Instruction::Inx { register } => self.execute_inx(&register),
            Instruction::Jc { address } =>
                self.execute_jc(address[1], address[0]),
            Instruction::Jm { address } =>
                self.execute_jm(address[1], address[0]),
            Instruction::Jnc { address } =>
                self.execute_jnc(address[1], address[0]),
            Instruction::Jnz { address } =>
                self.execute_jnz(address[1], address[0]),
            Instruction::Jmp { address } =>
                self.execute_jmp(address[1], address[0]),
            Instruction::Jp { address } =>
                self.execute_jp(address[1], address[0]),
            Instruction::Jpe { address } =>
                self.execute_jpe(address[1], address[0]),
            Instruction::Jpo { address } =>
                self.execute_jpo(address[1], address[0]),
            Instruction::Jz { address } =>
                self.execute_jz(address[1], address[0]),
            Instruction::Lda { address } =>
                self.execute_lda(address[1], address[0]),
            Instruction::Ldax { register } => self.execute_ldax(&register),
            Instruction::Lhld { address } =>
                self.execute_lhld(address[1], address[0]),
            Instruction::Lxi { register, low_byte, high_byte } =>
                self.execute_lxi(&register, high_byte, low_byte),
            Instruction::Mov { destiny, source } => self.execute_mov(&destiny, &source),
            Instruction::Mvi { source: Location::Memory, byte} => self.execute_mvi_to_memory(byte),
            Instruction::Mvi { source: Location::Register { register }, byte } =>
                self.save_to_single_register(byte, &register),
            Instruction::Noop => self.execute_noop(),
            Instruction::Pchl => self.execute_pchl(),
            Instruction::Pop { register } => self.execute_pop(&register),
            Instruction::Push { register } => self.execute_push(&register),
            Instruction::Ora { source: Location::Register { register } } =>
                self.execute_ora_by_register(&register),
            Instruction::Ora { source: Location::Memory } => self.execute_ora_by_memory(),
            Instruction::Ori { byte } => self.execute_ori(byte),
            Instruction::Out { byte } => self.execute_out(byte),
            Instruction::Ral => self.execute_ral(),
            Instruction::Rar => self.execute_rar(),
            Instruction::Rc => self.execute_rc(),
            Instruction::Ret => self.execute_ret(),
            Instruction::Rlc => self.execute_rlc(),
            Instruction::Rm => self.execute_rm(),
            Instruction::Rnc => self.execute_rnc(),
            Instruction::Rnz => self.execute_rnz(),
            Instruction::Rp => self.execute_rp(),
            Instruction::Rpe => self.execute_rpe(),
            Instruction::Rpo => self.execute_rpo(),
            Instruction::Rrc => self.execute_rrc(),
            Instruction::Rst { value } => self.execute_rst(value),
            Instruction::Rz => self.execute_rz(),
            Instruction::Sbb { source: Location::Register { register } } =>
                self.execute_sbb_by_register(&register),
            Instruction::Sbb { source: Location::Memory } => self.execute_sbb_by_memory(),
            Instruction::Sbi { byte } => self.execute_sbi(byte),
            Instruction::Shld { address } =>
                self.execute_shld(address[1], address[0]),
            Instruction::Sta { address } =>
                self.execute_sta(address[1], address[0]),
            Instruction::Stax { register } => self.execute_stax(&register),
            Instruction::Stc => self.execute_stc(),
            Instruction::Sphl => self.execute_sphl(),
            Instruction::Sub { source: Location::Register { register } } =>
                self.execute_sub_by_register(&register),
            Instruction::Sub { source: Location::Memory } => self.execute_sub_by_memory(),
            Instruction::Sui { byte } => self.execute_sui(byte),
            Instruction::Xchg => self.execute_xchg(),
            Instruction::Xra { source: Location::Register { register } } =>
                self.execute_xra_by_register(&register),
            Instruction::Xra { source: Location::Memory } => self.execute_xra_by_memory(),
            Instruction::Xri { byte } => self.execute_xri(byte),
            Instruction::Xthl => self.execute_xthl(),
        }
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.pc as usize;
        let to = min(from+3, self.memory.len());
        &(self.memory[from..to])
    }

    #[inline]
    fn can_run(&self, instruction: &Instruction) -> bool {
        match instruction {
            Instruction::Rst { value: _ } => true,
            _ if self.state == State::Running => true,
            _ => false,
        }
    }

    fn execute_noop(&self) {}
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, ROM_MEMORY_LIMIT, State};

    #[test]
    fn it_should_execute_instruction_when_running() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.state = State::Running;
        cpu.pc = 0;
        cpu.memory[0] = 0x00;
        cpu.execute();
        assert_eq!(cpu.pc, 0x01);
    }

    #[test]
    fn it_shouldnt_execute_instruction_when_stopped() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.state = State::Stopped;
        cpu.pc = 0;
        cpu.memory[0] = 0x00;
        cpu.execute();
        assert_eq!(cpu.pc, 0x00);
    }
}