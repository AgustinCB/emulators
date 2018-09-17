extern crate mos6502cpu;

use mos6502cpu::{AVAILABLE_MEMORY, Memory};
use nes::InputOutputDevice;

pub const ROM_SIZE: usize = 0x8000;

pub(crate) struct IORegister {
    pub(crate) current: u8,
    pub(crate) device: Option<Box<InputOutputDevice>>,
}

impl IORegister {
    pub(crate) fn new() -> IORegister {
        IORegister {
            current: 0,
            device: None,
        }
    }
    pub(crate) fn update(&mut self, new_current: u8) {
        if let Some(ref mut device) = self.device {
            self.current = device.write(new_current);
        } else {
            self.current = new_current;
        }
    }
    pub(crate) fn current(&self) -> u8 {
        if let Some(ref device) = self.device {
            device.read()
        } else {
            self.current
        }
    }
}

pub struct Ram {
    ram: [u8; 0x800],
    pub(crate) io_registers: Vec<IORegister>,
    expansion_rom: [u8; 0x1FE0],
    sram: [u8; 0x2000],
    rom: [u8; ROM_SIZE],
}

impl Ram {
    pub fn new(rom: [u8; ROM_SIZE]) -> Ram {
        let mut io_registers = Vec::with_capacity(0x28);
        for _ in 0..0x28 {
            io_registers.push(IORegister::new());
        }
        Ram {
            ram: [0; 0x800],
            expansion_rom: [0; 0x1E00],
            sram: [0; 0x2000],
            io_registers,
            rom
        }
    }

    fn set_in_ram(&mut self, index: u16, new_value: u8) {
        self.ram[index as usize % 0x800] = new_value;
    }

    fn set_in_io(&mut self, index: u16, new_current: u8) {
        if index < 0x4000 {
            let io_index = index - 0x2000;
            self.io_registers[io_index as usize % 8].update(new_current);
        } else {
            self.io_registers[index as usize - 0x4000 + 0x8].update(new_current);
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
            self.io_registers[io_index as usize % 8].current()
        } else {
            self.io_registers[index as usize - 0x4000 + 0x8].current()
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

impl Memory for Ram {
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
    fn len(&self) -> usize {
        AVAILABLE_MEMORY
    }
}

#[cfg(test)]
mod tests {
    use mos6502cpu::Memory;
    use ram::{Ram, ROM_SIZE};

    #[test]
    fn it_should_set_in_ram() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x0, 0x42);
        assert_eq!(memory.ram[0x0], 0x42);
    }

    #[test]
    fn it_should_set_in_ram_mirroring() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x800, 0x42);
        assert_eq!(memory.ram[0x0], 0x42);
    }

    #[test]
    fn it_should_set_in_io_registers() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x2000, 0x42);
        assert_eq!(memory.io_registers[0x0].current, 0x42);
        memory.set(0x4001, 0x42);
        assert_eq!(memory.io_registers[0x9].current, 0x42);
    }

    #[test]
    fn it_should_set_in_io_registers_mirroring() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x2008, 0x42);
        assert_eq!(memory.io_registers[0x0].current, 0x42);
    }

    #[test]
    fn it_should_set_in_expansion_rom() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x4020, 0x42);
        assert_eq!(memory.expansion_rom[0x0], 0x42);
    }

    #[test]
    fn it_should_set_in_sram() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x6000, 0x42);
        assert_eq!(memory.sram[0x0], 0x42);
    }

    #[test]
    fn it_shouldnt_set_in_rom() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.set(0x8000, 0x42);
        assert_eq!(memory.sram[0x0], 0x0);
    }

    #[test]
    fn it_should_get_from_ram() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.ram[0x0] = 0x42;
        assert_eq!(memory.get(0), 0x42);
    }

    #[test]
    fn it_should_get_from_ram_with_mirroring() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.ram[0x0] = 0x42;
        assert_eq!(memory.get(0), 0x42);
        assert_eq!(memory.get(0x0800), 0x42);
        assert_eq!(memory.get(0x1000), 0x42);
        assert_eq!(memory.get(0x1800), 0x42);
    }

    #[test]
    fn it_should_get_from_io_registers() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.io_registers[0x0].update(0x42);
        assert_eq!(memory.get(0x2000), 0x42);
        memory.io_registers[0x9].update(0x42);
        assert_eq!(memory.get(0x4001), 0x42);
    }

    #[test]
    fn it_should_get_from_io_registers_mirroring() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.io_registers[0x0].update(0x42);
        assert_eq!(memory.get(0x2000), 0x42);
        assert_eq!(memory.get(0x2008), 0x42);
        assert_eq!(memory.get(0x2010), 0x42);
    }

    #[test]
    fn it_should_get_from_expansion_rom() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.expansion_rom[0x0] = 0x42;
        assert_eq!(memory.get(0x4020), 0x42);
    }

    #[test]
    fn it_should_get_from_sram() {
        let mut memory = Ram::new([0; ROM_SIZE]);
        memory.sram[0x0] = 0x42;
        assert_eq!(memory.get(0x6000), 0x42);
    }

    #[test]
    fn it_should_get_from_rom() {
        let memory = Ram::new([0x42; ROM_SIZE]);
        assert_eq!(memory.get(0x8000), 0x42);
    }
}