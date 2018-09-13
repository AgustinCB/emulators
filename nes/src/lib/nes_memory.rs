extern crate mos6502cpu;

use mos6502cpu::{AVAILABLE_MEMORY, Memory};

pub const ROM_SIZE: usize = 0x8000;

pub struct NesMemory {
    ram: [u8; 0x800],
    io_registers: [u8; 28],
    expansion_rom: [u8; 0x1E00],
    sram: [u8; 0x2000],
    rom: [u8; ROM_SIZE],
}

impl NesMemory {
    pub fn new(rom: [u8; ROM_SIZE]) -> NesMemory {
        NesMemory {
            ram: [0; 0x800],
            io_registers: [0; 28],
            expansion_rom: [0; 0x1E00],
            sram: [0; 0x2000],
            rom
        }
    }

    fn set_in_ram(&mut self, index: u16, new_value: u8) {
        self.ram[index as usize % 0x800] = new_value;
    }

    fn set_in_io(&mut self, index: u16, new_value: u8) {
        if index < 0x4000 {
            let io_index = index - 0x2000;
            self.io_registers[io_index as usize % 8] = new_value;
        } else {
            self.io_registers[index as usize - 0x4000] = new_value;
        }
    }

    fn set_in_expansion_rom(&mut self, index: u16, new_value: u8) {
        self.expansion_rom[index as usize - 0x4020] = new_value;
    }

    fn set_in_sram(&mut self, index: u16, new_value: u8) {
        self.sram[index as usize - 0x6000] = new_value;
    }

    fn get_from_ram(&self, index: u16) -> u8 {
        self.ram[index as usize % 0x800]
    }

    fn get_from_io(&self, index: u16) -> u8 {
        if index < 0x4000 {
            let io_index = index - 0x2000;
            self.io_registers[io_index as usize % 8]
        } else {
            self.io_registers[index as usize - 0x4000]
        }
    }

    fn get_from_expansion_rom(&self, index: u16) -> u8 {
        self.expansion_rom[index as usize - 0x4020]
    }

    fn get_from_sram(&self, index: u16) -> u8 {
        self.sram[index as usize - 0x6000]
    }

    fn get_from_rom(&self, index: u16) -> u8 {
        self.rom[index as usize - 0x8000]
    }
}

impl Memory for NesMemory {
    fn set(&mut self, index: u16, new_value: u8) {
        if index < 0x2000 {
            self.set_in_ram(index, new_value);
        } else if index < 0x4020 {
            self.set_in_io(index, new_value);
        } else if index < 0x6000 {
            self.set_in_expansion_rom(index, new_value);
        } else if index < 0x8000 {
            self.set_in_sram(index, new_value);
        }
    }
    fn get(&self, index: u16) -> u8 {
        if index < 0x2000 {
            self.get_from_ram(index)
        } else if index < 0x4020 {
            self.get_from_io(index)
        } else if index < 0x6000 {
            self.get_from_expansion_rom(index)
        } else if index < 0x8000 {
            self.get_from_sram(index)
        } else {
            self.get_from_rom(index)
        }
    }
    // Slice can only happen in the ROM
    fn slice(&self, from: usize, to: usize) -> &[u8] {
        &self.rom[(from - 0x8000)..(to - 0x8000)]
    }
    fn len(&self) -> usize {
        AVAILABLE_MEMORY
    }
}