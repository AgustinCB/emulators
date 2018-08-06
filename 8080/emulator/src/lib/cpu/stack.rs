use cpu::cpu::Cpu;
use disassembler_8080::RegisterType;

impl Cpu {
    pub(crate) fn execute_push(&mut self, register: &RegisterType) {
        let sp = self.get_current_sp_value() as usize;
        let (first_byte, second_byte) = match register {
            RegisterType::B =>
                (self.get_current_single_register_value(&RegisterType::B), self.get_current_single_register_value(&RegisterType::C)),
            RegisterType::D =>
                (self.get_current_single_register_value(&RegisterType::D), self.get_current_single_register_value(&RegisterType::E)),
            RegisterType::H =>
                (self.get_current_single_register_value(&RegisterType::H), self.get_current_single_register_value(&RegisterType::L)),
            RegisterType::Psw =>
                (self.get_current_a_value(), self.get_current_flags_byte()),
            _ => panic!("{} is not a valid register for push!", register.to_string()),
        };
        self.memory[sp-1] = first_byte;
        self.memory[sp-2] = second_byte;
        self.save_to_double_register((sp-2) as u16, &RegisterType::Sp);
    }

    pub(crate) fn execute_pop(&mut self, register: &RegisterType) {
        let sp = self.get_current_sp_value() as usize;
        let first_byte = self.memory[sp+1];
        let second_byte = self.memory[sp];
        match register {
            RegisterType::B => {
                self.save_to_single_register(first_byte, &RegisterType::B);
                self.save_to_single_register(first_byte, &RegisterType::C);
            },
            RegisterType::D => {
                self.save_to_single_register(first_byte, &RegisterType::D);
                self.save_to_single_register(first_byte, &RegisterType::E);
            },
            RegisterType::H => {
                self.save_to_single_register(first_byte, &RegisterType::H);
                self.save_to_single_register(first_byte, &RegisterType::L);
            },
            RegisterType::Psw => {
                self.save_to_a(first_byte);
                self.set_flags_byte(second_byte);
            },
            _ => panic!("{} is not a valid register for push!", register.to_string()),
        };
        self.save_to_double_register((sp+2) as u16, &RegisterType::Sp);
    }
}