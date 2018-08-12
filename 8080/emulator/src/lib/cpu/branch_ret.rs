use cpu::cpu::{Cpu, RegisterType};

impl<'a> Cpu<'a> {
    pub(crate) fn execute_rc(&mut self) {
        if self.flags.carry {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_ret(&mut self) {
        self.perform_ret();
    }

    pub(crate) fn execute_rm(&mut self) {
        if self.flags.sign {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_rnc(&mut self) {
        if !self.flags.carry {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_rnz(&mut self) {
        if !self.flags.zero {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_rpe(&mut self) {
        if self.flags.parity {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_rpo(&mut self) {
        if !self.flags.parity {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_rp(&mut self) {
        if !self.flags.sign {
            self.perform_ret();
        }
    }

    pub(crate) fn execute_rz(&mut self) {
        if self.flags.zero {
            self.perform_ret();
        }
    }

    #[inline]
    fn perform_ret(&mut self) {
        let sp = self.get_current_sp_value() as usize;
        let high_byte = self.memory[sp + 1];
        let low_byte = self.memory[sp];
        self.perform_jump(high_byte, low_byte);
        self.save_to_double_register((sp + 2) as u16, &RegisterType::Sp);
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, RegisterType, ROM_MEMORY_LIMIT};
    use cpu::instruction::Instruction;

    #[test]
    fn it_should_execute_rc_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Rc);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rc_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Rc);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_ret() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.execute_instruction(Instruction::Ret);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_should_execute_rm_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.sign = true;
        cpu.execute_instruction(Instruction::Rm);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rm_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.sign = false;
        cpu.execute_instruction(Instruction::Rm);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_rnc_if_carry_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.carry = false;
        cpu.execute_instruction(Instruction::Rnc);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rnc_if_carry_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.carry = true;
        cpu.execute_instruction(Instruction::Rnc);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_rnz_if_zero_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.zero = false;
        cpu.execute_instruction(Instruction::Rnz);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rnz_if_zero_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.zero = true;
        cpu.execute_instruction(Instruction::Rnz);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_rp_if_sign_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.sign = false;
        cpu.execute_instruction(Instruction::Rp);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rp_if_sign_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.sign = true;
        cpu.execute_instruction(Instruction::Rp);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_rpe_if_parity_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.parity = true;
        cpu.execute_instruction(Instruction::Rpe);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rpe_if_parity_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.parity = false;
        cpu.execute_instruction(Instruction::Rpe);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_rpo_if_parity_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.parity = false;
        cpu.execute_instruction(Instruction::Rpo);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rpo_if_parity_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.parity = true;
        cpu.execute_instruction(Instruction::Rpo);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }

    #[test]
    fn it_should_execute_rz_if_zero_is_set() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.zero = true;
        cpu.execute_instruction(Instruction::Rz);
        assert_eq!(cpu.pc, 0x2c03);
        assert_eq!(cpu.get_current_sp_value(), 2);
    }

    #[test]
    fn it_shouldnt_execute_rz_if_zero_is_reset() {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        cpu.save_to_double_register(0, &RegisterType::Sp);
        cpu.memory[0] = 0x03;
        cpu.memory[1] = 0x2c;
        cpu.pc = 0x2442;
        cpu.flags.zero = false;
        cpu.execute_instruction(Instruction::Rz);
        assert_eq!(cpu.pc, 0x2442);
        assert_eq!(cpu.get_current_sp_value(), 0);
    }
}