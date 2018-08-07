extern crate disassembler_8080;

use self::disassembler_8080::{Instruction, Location};
use cpu::cpu::Cpu;
use std::cmp::min;

impl Cpu {
    pub fn execute(&mut self) {
        let instruction = Instruction::from_bytes(self.get_next_instruction_bytes());
        self.pc += instruction.size() as u16;
        self.execute_instruction(instruction);
    }

    pub fn execute_instruction(&mut self, instruction: Instruction) {
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
            Instruction::Cma => self.execute_cma(),
            Instruction::Cmc => self.execute_cmc(),
            Instruction::Cmp { source: Location::Register { register } } =>
                self.execute_cmp_by_register(&register),
            Instruction::Cmp { source: Location::Memory } => self.execute_cmp_by_memory(),
            Instruction::Cpi { byte } => self.execute_cpi(byte),
            Instruction::Daa => self.execute_daa(),
            Instruction::Dad { register } => self.execute_dad(&register),
            Instruction::Dcr { source: Location::Register { register } } =>
                self.execute_dcr_by_register(&register),
            Instruction::Dcr { source: Location::Memory } => self.execute_dcr_by_memory(),
            Instruction::Dcx { register } => self.execute_dcx(&register),
            Instruction::Inr { source: Location::Register { register } } =>
                self.execute_inr_by_register(&register),
            Instruction::Inr { source: Location::Memory } => self.execute_inr_by_memory(),
            Instruction::Inx { register } => self.execute_inx(&register),
            Instruction::Jmp { address } =>
                self.execute_jmp(address[1], address[0]),
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
            Instruction::Pchl => self.execute_pchl(),
            Instruction::Pop { register } => self.execute_pop(&register),
            Instruction::Push { register } => self.execute_push(&register),
            Instruction::Ora { source: Location::Register { register } } =>
                self.execute_ora_by_register(&register),
            Instruction::Ora { source: Location::Memory } => self.execute_ora_by_memory(),
            Instruction::Ori { byte } => self.execute_ori(byte),
            Instruction::Ral => self.execute_ral(),
            Instruction::Rar => self.execute_rar(),
            Instruction::Rlc => self.execute_rlc(),
            Instruction::Rrc => self.execute_rrc(),
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
            _ => println!("Execute: {}", instruction.to_string()),
        }
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.pc as usize;
        let to = min(from+3, self.memory.len());
        &(self.memory[from..to])
    }
}
