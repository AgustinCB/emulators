use cpu::cpu::{Cpu, RegisterType};

impl<'a> Cpu<'a> {
    pub(crate) fn execute_push(&mut self, register: &RegisterType) {
        let sp = self.get_current_sp_value() as usize;
        let (first_byte, second_byte) = match register {
            RegisterType::B =>
                (self.get_current_single_register_value(&RegisterType::B),
                 self.get_current_single_register_value(&RegisterType::C)),
            RegisterType::D =>
                (self.get_current_single_register_value(&RegisterType::D),
                 self.get_current_single_register_value(&RegisterType::E)),
            RegisterType::H =>
                (self.get_current_single_register_value(&RegisterType::H),
                 self.get_current_single_register_value(&RegisterType::L)),
            RegisterType::Psw =>
                (self.get_current_a_value(), self.get_current_flags_byte()),
            _ => panic!("{} is not a valid register for push!", register.to_string()),
        };
        self.memory[sp-1].set(first_byte);
        self.memory[sp-2].set(second_byte);
        self.save_to_double_register((sp-2) as u16, &RegisterType::Sp);
    }

    pub(crate) fn execute_pop(&mut self, register: &RegisterType) {
        let sp = self.get_current_sp_value() as usize;
        let first_byte = self.memory[sp+1].get();
        let second_byte = self.memory[sp].get();
        match register {
            RegisterType::B => {
                self.save_to_single_register(first_byte, &RegisterType::B);
                self.save_to_single_register(second_byte, &RegisterType::C);
            },
            RegisterType::D => {
                self.save_to_single_register(first_byte, &RegisterType::D);
                self.save_to_single_register(second_byte, &RegisterType::E);
            },
            RegisterType::H => {
                self.save_to_single_register(first_byte, &RegisterType::H);
                self.save_to_single_register(second_byte, &RegisterType::L);
            },
            RegisterType::Psw => {
                self.save_to_a(first_byte);
                self.set_flags_byte(second_byte);
            },
            _ => panic!("{} is not a valid register for push!", register.to_string()),
        };
        self.save_to_double_register((sp+2) as u16, &RegisterType::Sp);
    }

    #[inline]
    fn get_current_flags_byte(&self) -> u8 {
        (self.flags.zero as u8) |
            (self.flags.sign as u8) << 1 |
            (self.flags.parity as u8) << 2 |
            (self.flags.carry as u8) << 3 |
            (self.flags.auxiliary_carry as u8) << 4
    }

    #[inline]
    fn set_flags_byte(&mut self, byte: u8) {
        self.flags.zero = (byte & 0x01) == 0x01;
        self.flags.sign = (byte & 0x02) == 0x02;
        self.flags.parity = (byte & 0x04) == 0x04;
        self.flags.carry = (byte & 0x08) == 0x08;
        self.flags.auxiliary_carry = (byte & 0x10) == 0x10;
    }
}

#[cfg(test)]
mod tests {
    use cpu::cpu::{Cpu, RegisterType, ROM_MEMORY_LIMIT};
    use cpu::instruction::Instruction;

    fn get_pop_ready_cpu<'a>() -> Cpu<'a> {
        let mut memory = [0; ROM_MEMORY_LIMIT];
        memory[0x1239] = 0x3d;
        memory[0x123A] = 0x93;
        let mut cpu = Cpu::new(memory);
        cpu.save_to_double_register(0x1239, &RegisterType::Sp);
        cpu
    }

    fn get_push_ready_cpu(register: &RegisterType) -> Cpu {
        let mut cpu = Cpu::new([0; ROM_MEMORY_LIMIT]);
        match register {
            RegisterType::B => {
                cpu.save_to_single_register(0x8f, &RegisterType::B);
                cpu.save_to_single_register(0x9d, &RegisterType::C);
            },
            RegisterType::D => {
                cpu.save_to_single_register(0x8f, &RegisterType::D);
                cpu.save_to_single_register(0x9d, &RegisterType::E);
            },
            RegisterType::H => {
                cpu.save_to_single_register(0x8f, &RegisterType::H);
                cpu.save_to_single_register(0x9d, &RegisterType::L);
            }
            RegisterType::Psw => {
                cpu.save_to_single_register(0x8f, &RegisterType::A);
                cpu.flags.zero = true;
                cpu.flags.sign = false;
                cpu.flags.parity = true;
                cpu.flags.carry = true;
                cpu.flags.auxiliary_carry = true;
            }
            _ => panic!("Register {} is not an argument for PUSH.", register.to_string()),
        }
        cpu.save_to_double_register(0x3A2C, &RegisterType::Sp);
        cpu
    }

    #[test]
    fn it_should_pop_from_stack_to_b() {
        let mut cpu = get_pop_ready_cpu();
        cpu.execute_instruction(Instruction::Pop { register: RegisterType::B });
        assert_eq!(cpu.get_current_bc_value(), 0x933d);
        assert_eq!(cpu.get_current_sp_value(), 0x123b);
    }

    #[test]
    fn it_should_pop_from_stack_to_d() {
        let mut cpu = get_pop_ready_cpu();
        cpu.execute_instruction(Instruction::Pop { register: RegisterType::D });
        assert_eq!(cpu.get_current_de_value(), 0x933d);
        assert_eq!(cpu.get_current_sp_value(), 0x123b);
    }

    #[test]
    fn it_should_pop_from_stack_to_h() {
        let mut cpu = get_pop_ready_cpu();
        cpu.execute_instruction(Instruction::Pop { register: RegisterType::H });
        assert_eq!(cpu.get_current_hl_value(), 0x933d);
        assert_eq!(cpu.get_current_sp_value(), 0x123b);
    }

    #[test]
    fn it_should_pop_from_stack_to_a_and_flags() {
        let mut cpu = get_pop_ready_cpu();
        cpu.execute_instruction(Instruction::Pop { register: RegisterType::Psw });
        assert_eq!(cpu.get_current_a_value(), 0x93);
        assert_eq!(cpu.get_current_sp_value(), 0x123b);
        assert!(cpu.flags.zero);
        assert!(!cpu.flags.sign);
        assert!(cpu.flags.parity);
        assert!(cpu.flags.carry);
        assert!(cpu.flags.auxiliary_carry);
    }

    #[test]
    fn it_should_push_from_stack_to_b() {
        let mut cpu = get_push_ready_cpu(&RegisterType::B);
        cpu.execute_instruction(Instruction::Push { register: RegisterType::B });
        assert_eq!(cpu.memory[0x3a2b].get(), 0x8f);
        assert_eq!(cpu.memory[0x3a2a].get(), 0x9d);
        assert_eq!(cpu.get_current_sp_value(), 0x3A2A);
    }

    #[test]
    fn it_should_push_from_stack_to_d() {
        let mut cpu = get_push_ready_cpu(&RegisterType::D);
        cpu.execute_instruction(Instruction::Push { register: RegisterType::D });
        assert_eq!(cpu.memory[0x3a2b].get(), 0x8f);
        assert_eq!(cpu.memory[0x3a2a].get(), 0x9d);
        assert_eq!(cpu.get_current_sp_value(), 0x3A2A);
    }

    #[test]
    fn it_should_push_from_stack_to_h() {
        let mut cpu = get_push_ready_cpu(&RegisterType::H);
        cpu.execute_instruction(Instruction::Push { register: RegisterType::H });
        assert_eq!(cpu.memory[0x3a2b].get(), 0x8f);
        assert_eq!(cpu.memory[0x3a2a].get(), 0x9d);
        assert_eq!(cpu.get_current_sp_value(), 0x3A2A);
    }

    #[test]
    fn it_should_push_from_stack_to_a_and_flags() {
        let mut cpu = get_push_ready_cpu(&RegisterType::Psw);
        cpu.execute_instruction(Instruction::Push { register: RegisterType::Psw });
        assert_eq!(cpu.memory[0x3a2b].get(), 0x8f);
        assert_eq!(cpu.memory[0x3a2a].get(), 0x1d);
        assert_eq!(cpu.get_current_sp_value(), 0x3A2A);
    }
}