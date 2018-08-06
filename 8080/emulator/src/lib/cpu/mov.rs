use cpu::cpu::Cpu;
use disassembler_8080::RegisterType;
use disassembler_8080::Location;

impl Cpu {
    pub(crate) fn execute_ldax(&mut self, register: &RegisterType) {
        let source_address = match register {
            RegisterType::B => self.get_current_bc_value(),
            RegisterType::D => self.get_current_de_value(),
            _ => panic!("Register {} is not a valid input of LDAX", register.to_string()),
        } as usize;
        let value = self.memory[source_address];
        self.save_to_a(value);
    }

    pub(crate) fn execute_lxi(&mut self, register_type: &RegisterType, high_byte: u8, low_byte: u8) {
        match register_type {
            RegisterType::B => {
                self.save_to_single_register(high_byte, &RegisterType::B);
                self.save_to_single_register(low_byte, &RegisterType::C);
            },
            RegisterType::D => {
                self.save_to_single_register(high_byte, &RegisterType::D);
                self.save_to_single_register(low_byte, &RegisterType::E);
            },
            RegisterType::H => {
                self.save_to_single_register(high_byte, &RegisterType::H);
                self.save_to_single_register(low_byte, &RegisterType::L);
            },
            RegisterType::Sp =>
                self.save_to_double_register((high_byte as u16) << 8 | (low_byte as u16), &RegisterType::Sp),
            _ => panic!("Register {} is not a valid input of LXI", register_type.to_string()),
        }
    }

    #[inline]
    pub(crate) fn execute_mov(&mut self, destiny: &Location, source: &Location) {
        match (destiny, source) {
            (Location::Register { register: destiny }, Location::Register { register: source }) =>
                self.execute_mov_register_to_register(&destiny, &source),
            (Location::Register { register: destiny }, Location::Memory) =>
                self.execute_mov_memory_to_register(&destiny),
            (Location::Memory, Location::Register { register: source }) =>
                self.execute_mov_register_to_memory(&source),
            (Location::Memory, Location::Memory) =>
                panic!("MOV (HL),(HL) can't happen!")
        }
    }

    #[inline]
    pub(crate) fn execute_mvi_to_memory(&mut self, byte: u8) {
        let address = self.get_current_hl_value();
        self.memory[address as usize] = byte;
    }

    pub(crate) fn execute_sphl(&mut self) {
        let hl = self.get_current_hl_value();
        self.save_to_double_register(hl, &RegisterType::Sp);
    }

    pub(crate) fn execute_stax(&mut self, register: &RegisterType) {
        let value = self.get_current_a_value();
        let destiny_address = match register {
            RegisterType::B => self.get_current_bc_value(),
            RegisterType::D => self.get_current_de_value(),
            _ => panic!("Register {} is not a valid input of STAX", register.to_string()),
        } as usize;
        self.memory[destiny_address] = value;
    }

    pub(crate) fn execute_xchg(&mut self) {
        let d_value = self.get_current_single_register_value(&RegisterType::D);
        let e_value = self.get_current_single_register_value(&RegisterType::E);
        let h_value = self.get_current_single_register_value(&RegisterType::H);
        let l_value = self.get_current_single_register_value(&RegisterType::L);
        self.save_to_single_register(h_value, &RegisterType::D);
        self.save_to_single_register(l_value, &RegisterType::E);
        self.save_to_single_register(d_value, &RegisterType::H);
        self.save_to_single_register(e_value, &RegisterType::L);
    }

    pub(crate) fn execute_xthl(&mut self) {
        let sp = self.get_current_sp_value() as usize;
        let first_byte = self.memory[sp+1];
        let second_byte = self.memory[sp];
        let h_value = self.get_current_single_register_value(&RegisterType::H);
        let l_value = self.get_current_single_register_value(&RegisterType::L);
        self.save_to_single_register(first_byte, &RegisterType::H);
        self.save_to_single_register(second_byte, &RegisterType::L);
        self.memory[sp+1] = h_value;
        self.memory[sp] = l_value;
    }

    #[inline]
    fn execute_mov_register_to_register(&mut self, destiny: &RegisterType, source: &RegisterType) {
        let source_value = self.get_current_single_register_value(source);
        self.save_to_single_register(source_value, destiny);
    }

    #[inline]
    fn execute_mov_memory_to_register(&mut self, destiny: &RegisterType) {
        let source_value = self.get_value_in_memory_at_hl();
        self.save_to_single_register(source_value, destiny);
    }

    #[inline]
    fn execute_mov_register_to_memory(&mut self, source: &RegisterType) {
        let source_value = self.get_current_single_register_value(source);
        self.set_value_in_memory_at_hl(source_value);
    }
}
