use bit_utils::two_bytes_to_word;
use cpu::{Cpu, Cycles, Instruction};
use failure::{Error, Fail};
use std::cmp::min;
use {CpuResult, Mos6502Instruction};
use super::instruction::{Mos6502InstructionCode, AddressingMode};

pub const AVAILABLE_MEMORY: usize = 0x10000;
const ZERO_PAGE_START: usize = 0;
const ZERO_PAGE_END: usize = 0x100;
const STACK_PAGE_START: usize = 0x100;
const STACK_PAGE_END: usize = 0x200;
pub(crate) const INTERRUPT_HANDLERS_START: usize = 0xFFFA;
const INTERRUPT_HANDLERS_END: usize = 0x10000;

#[derive(Debug, Fail)]
pub enum CpuError {
    #[fail(display = "Attempt to access reserved memory. 0x0000-0x0200 and 0xFFFA to 0x10000 are reserved.")]
    ReservedMemory,
    #[fail(display = "Attempt to use invalid addressing mode.")]
    InvalidAddressingMode,
}

pub(crate) struct ProcessorStatus {
    pub(crate) negative: bool,
    pub(crate) overflow: bool,
    pub(crate) break_flag: bool,
    pub(crate) decimal: bool,
    pub(crate) interrupt_disable: bool,
    pub(crate) zero: bool,
    pub(crate) carry: bool,
}

impl ProcessorStatus {
    fn new() -> ProcessorStatus {
        ProcessorStatus {
            negative: false,
            overflow: false,
            break_flag: false,
            decimal: false,
            interrupt_disable: false,
            zero: false,
            carry: false,
        }
    }

    pub(crate) fn from_byte(byte: u8) -> ProcessorStatus {
        ProcessorStatus {
            negative: (byte & 0x80) > 0,
            overflow: (byte & 0x40) > 0,
            break_flag: (byte & 0x10) > 0,
            decimal: (byte & 0x08) > 0,
            interrupt_disable: (byte & 0x04) > 0,
            zero: (byte & 0x02) > 0,
            carry: (byte & 0x01) > 0,
        }
    }

    pub(crate) fn to_byte(&self) -> u8 {
        ((self.negative as u8) << 7) |
            ((self.overflow as u8) << 6) |
            0x20 |
            ((self.break_flag as u8) << 4) |
            ((self.decimal as u8) << 3) |
            ((self.interrupt_disable as u8) << 2) |
            ((self.zero as u8) << 1) |
            (self.carry as u8)
    }
}

pub(crate) struct RegisterSet {
    pub(crate) pc: u16,
    pub(crate) s: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) a: u8,
    pub(crate) p: ProcessorStatus,
}

impl RegisterSet {
    fn new() -> RegisterSet {
        RegisterSet {
            pc: 0,
            s: 0xff,
            x: 0,
            y: 0,
            a: 0,
            p: ProcessorStatus::new(),
        }
    }
}

pub struct Mos6502Cpu {
    pub(crate) memory: [u8; AVAILABLE_MEMORY],
    pub(crate) registers: RegisterSet,
    pub(crate) page_crossed: bool,
}

impl Mos6502Cpu {
    pub fn new(memory: [u8; AVAILABLE_MEMORY]) -> Mos6502Cpu {
        Mos6502Cpu {
            memory,
            registers: RegisterSet::new(),
            page_crossed: false,
        }
    }

    #[inline]
    fn execute_nop(&self) {
    }

    pub fn get_memory_slice(&mut self, from: usize, to: usize) -> Result<&[u8], CpuError> {
        if (from >= ZERO_PAGE_START && from < ZERO_PAGE_END) ||
            (from >= STACK_PAGE_START && from < STACK_PAGE_END) ||
            (from >= INTERRUPT_HANDLERS_START && from < INTERRUPT_HANDLERS_END) {
            Err(CpuError::ReservedMemory)
        } else if (to >= ZERO_PAGE_START && to < ZERO_PAGE_END) ||
            (to >= STACK_PAGE_START && to < STACK_PAGE_END) ||
            (to >= INTERRUPT_HANDLERS_START && to < INTERRUPT_HANDLERS_END) {
            Err(CpuError::ReservedMemory)
        } else if (from < ZERO_PAGE_START && to >= ZERO_PAGE_END) ||
            (from < STACK_PAGE_START && to >= STACK_PAGE_END) ||
            (from < INTERRUPT_HANDLERS_START && to >= INTERRUPT_HANDLERS_END) {
            Err(CpuError::ReservedMemory)
        } else {
            Ok(&mut self.memory[from..to])
        }
    }

    pub(crate) fn get_address_from_addressing_mode(
        &self, addressing_mode: &AddressingMode) -> Result<u16, CpuError> {
        match addressing_mode {
            AddressingMode::Indirect { high_byte, low_byte } => {
                let indirect_address = two_bytes_to_word(*high_byte, *low_byte);
                let (low_byte, high_byte) =
                    (self.memory[indirect_address as usize], self.memory[indirect_address as usize+1]);
                Ok(two_bytes_to_word(high_byte, low_byte))
            },
            AddressingMode::Absolute { high_byte, low_byte } => {
                Ok(two_bytes_to_word(*high_byte, *low_byte))
            },
            AddressingMode::AbsoluteIndexedX { low_byte, high_byte } => {
                let address = two_bytes_to_word(*high_byte, *low_byte) as u16;
                Ok(address + self.registers.x as u16)
            },
            AddressingMode::AbsoluteIndexedY { low_byte, high_byte } => {
                let address = two_bytes_to_word(*high_byte, *low_byte) as u16;
                Ok(address + self.registers.y as u16)
            },
            AddressingMode::ZeroPage { byte } => Ok(*byte as u16),
            AddressingMode::ZeroPageIndexedX { byte } => {
                let x = self.registers.x as u16;
                Ok((x + *byte as u16) & 0x00ff)
            },
            AddressingMode::ZeroPageIndexedY { byte } => {
                let y = self.registers.y as u16;
                Ok((y + *byte as u16) & 0x00ff)
            },
            AddressingMode::IndexedIndirect { byte } => {
                let indirect_address = ((*byte as u16 + self.registers.x as u16) as u8) as usize;
                let (low_byte, high_byte) =
                    (self.memory[indirect_address], self.memory[indirect_address+1]);
                Ok(two_bytes_to_word(high_byte, low_byte))
            },
            AddressingMode::IndirectIndexed { byte } => {
                let (low_byte, high_byte) =
                    (self.memory[*byte as usize], self.memory[*byte as usize + 1]);
                Ok(two_bytes_to_word(high_byte, low_byte) + self.registers.y as u16)
            },
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn get_value_from_addressing_mode(&self, addressing_mode: &AddressingMode)
        -> Result<u8, CpuError> {
        match addressing_mode {
            AddressingMode::Accumulator => Ok(self.registers.a),
            AddressingMode::Immediate { byte } => Ok(*byte),
            AddressingMode::ZeroPage { byte } => Ok(self.memory[*byte as usize]),
            AddressingMode::ZeroPageIndexedX { byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            AddressingMode::ZeroPageIndexedY { byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            AddressingMode::Absolute { high_byte: _, low_byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            AddressingMode::AbsoluteIndexedX { high_byte: _, low_byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            AddressingMode::AbsoluteIndexedY { high_byte: _, low_byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            AddressingMode::IndexedIndirect { byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            AddressingMode::IndirectIndexed { byte: _ } =>
                Ok(self.memory[self.get_address_from_addressing_mode(addressing_mode)? as usize]),
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn set_value_to_addressing_mode(&mut self, addressing_mode: &AddressingMode, value: u8)
        -> CpuResult {
        match addressing_mode {
            AddressingMode::Accumulator => Ok(self.registers.a = value),
            AddressingMode::ZeroPage { byte } => Ok(self.memory[*byte as usize] = value),
            AddressingMode::ZeroPageIndexedX { byte } => {
                let x = self.registers.x as u16;
                let address = (x + *byte as u16) as u8;
                self.memory[address as usize] = value;
                Ok(())
            },
            AddressingMode::ZeroPageIndexedY { byte } => {
                let y = self.registers.y as u16;
                let address = (y + *byte as u16) as u8;
                self.memory[address as usize] = value;
                Ok(())
            },
            AddressingMode::Absolute { high_byte, low_byte } => {
                let address = two_bytes_to_word(*high_byte, *low_byte) as usize;
                self.memory[address] = value;
                Ok(())
            },
            AddressingMode::AbsoluteIndexedX { high_byte, low_byte } => {
                let x = self.registers.x;
                let address = two_bytes_to_word(*high_byte, *low_byte) as usize;
                self.memory[address + x as usize] = value;
                self.update_page_crossed_status(address as u16, address as u16 + x as u16);
                Ok(())
            },
            AddressingMode::AbsoluteIndexedY { high_byte, low_byte } => {
                let y = self.registers.y;
                let address = two_bytes_to_word(*high_byte, *low_byte) as usize;
                self.memory[address + y as usize] = value;
                self.update_page_crossed_status(address as u16, address as u16 + y as u16);
                Ok(())
            },
            AddressingMode::IndexedIndirect { byte } => {
                let indirect_address = ((*byte as u16 + self.registers.x as u16) as u8) as usize;
                let (low_byte, high_byte) =
                    (self.memory[indirect_address], self.memory[indirect_address+1]);
                self.memory[two_bytes_to_word(high_byte, low_byte) as usize] = value;
                Ok(())
            },
            AddressingMode::IndirectIndexed { byte } => {
                let y = self.registers.y;
                let (low_byte, high_byte) =
                    (self.memory[*byte as usize], self.memory[*byte as usize + 1]);
                let indirect_address = two_bytes_to_word(high_byte, low_byte);
                let direct_address = indirect_address + y as u16;
                self.update_page_crossed_status(indirect_address, direct_address);
                self.memory[direct_address as usize] = value;
                Ok(())
            },
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    #[inline]
    pub(crate) fn update_page_crossed_status(&mut self, original: u16, new: u16) {
        self.page_crossed = (original & 0xff00) == (new & 0xff00);
    }

    #[inline]
    pub(crate) fn push(&mut self, value: u8) {
        self.memory[self.registers.s as usize] = value;
        self.registers.s -= 1;
    }

    #[inline]
    pub(crate) fn pull(&mut self) -> u8 {
        self.registers.s += 1;
        let value = self.memory[self.registers.s as usize];
        value
    }
}

impl Cpu<u8, Mos6502Instruction, CpuError> for Mos6502Cpu {
    fn get_cycles_for_instruction(&mut self, instruction: &Mos6502Instruction) -> Result<u8, Error> {
        let cycles = match instruction.get_cycles()? {
            Cycles::Single(cycles) => cycles,
            Cycles::OneCondition { not_met, met } =>
                self.get_cycles_from_one_condition(instruction, not_met, met),
            Cycles::TwoConditions { not_met, first_met, second_met } =>
                self.get_cycles_from_two_conditions(instruction, not_met, first_met, second_met),
        };
        self.page_crossed = false;
        Ok(cycles)
    }

    fn execute_instruction(&mut self, instruction: &Mos6502Instruction) -> Result<(), Error> {
        if !self.can_run(&instruction) {
            return Ok(());
        }
        match instruction.instruction {
            Mos6502InstructionCode::Adc => self.execute_adc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::And => self.execute_and(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Asl => self.execute_asl(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bcc => self.execute_bcc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bcs => self.execute_bcs(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Beq => self.execute_beq(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bit => self.execute_bit(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bmi => self.execute_bmi(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bne => self.execute_bne(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bpl => self.execute_bpl(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Brk => self.execute_brk(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bvc => self.execute_bvc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Bvs => self.execute_bvs(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Clc => self.execute_clc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Cld => self.execute_cld(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Cli => self.execute_cli(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Clv => self.execute_clv(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Cmp => self.execute_cmp(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Cpx => self.execute_cpx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Cpy => self.execute_cpy(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Dec => self.execute_dec(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Dex => self.execute_dex(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Dey => self.execute_dey(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Eor => self.execute_eor(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Inc => self.execute_inc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Inx => self.execute_inx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Iny => self.execute_iny(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Jmp => self.execute_jmp(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Jsr => self.execute_jsr(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Lda => self.execute_lda(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Ldx => self.execute_ldx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Ldy => self.execute_ldy(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Lsr => self.execute_lsr(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Nop => self.execute_nop(),
            Mos6502InstructionCode::Ora => self.execute_ora(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Pha => self.execute_pha(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Php => self.execute_php(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Pla => self.execute_pla(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Plp => self.execute_plp(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rol => self.execute_rol(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Ror => self.execute_ror(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rti => self.execute_rti(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rts => self.execute_rts(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sbc => self.execute_sbc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sec => self.execute_sec(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sed => self.execute_sed(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sei => self.execute_sei(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sta => self.execute_sta(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Stx => self.execute_stx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sty => self.execute_sty(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tax => self.execute_tax(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tay => self.execute_tay(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tsx => self.execute_tsx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Txa => self.execute_txa(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Txs => self.execute_txs(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tya => self.execute_tya(&instruction.addressing_mode)?,
        };
        Ok(())
    }

    fn get_next_instruction_bytes(&self) -> &[u8] {
        let from = self.registers.pc as usize;
        let to = min(from+3, self.memory.len());
        &self.memory[from..to]
    }

    fn can_run(&self, _: &Mos6502Instruction) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.registers.pc >= AVAILABLE_MEMORY as u16
    }

    fn increase_pc(&mut self, steps: u8) {
        self.registers.pc += steps as u16
    }

    // TODO: Remove this panic and use Result<u8, CpuError) as a return instead
    fn get_cycles_from_one_condition
        (&self, instruction: &Mos6502Instruction, not_met: u8, met: u8) -> u8 {
        macro_rules! page_crossed_condition {
            () => {
                if self.page_crossed { met } else { not_met }
            }
        }
        match instruction.instruction {
            Mos6502InstructionCode::Adc => page_crossed_condition!(),
            Mos6502InstructionCode::And => page_crossed_condition!(),
            Mos6502InstructionCode::Cmp => page_crossed_condition!(),
            Mos6502InstructionCode::Lda => page_crossed_condition!(),
            Mos6502InstructionCode::Ldx => page_crossed_condition!(),
            Mos6502InstructionCode::Ldy => page_crossed_condition!(),
            Mos6502InstructionCode::Nop => page_crossed_condition!(),
            Mos6502InstructionCode::Ora => page_crossed_condition!(),
            Mos6502InstructionCode::Sbc => page_crossed_condition!(),
            _ => panic!("This instruction doesn't have conditional cycles."),
        }
    }

    // TODO: Remove this panic and use Result<u8, CpuError) as a return instead
    fn get_cycles_from_two_conditions
        (&self, instruction: &Mos6502Instruction, not_met: u8, first_met: u8, second_met: u8) -> u8 {
        macro_rules! bicondition {
            ($condition:expr) => {
                if $condition {
                    if self.page_crossed { second_met } else { first_met }
                } else {
                    not_met
                }
            }
        }
        match instruction.instruction {
            Mos6502InstructionCode::Bcc => bicondition!(!self.registers.p.carry),
            Mos6502InstructionCode::Bcs => bicondition!(self.registers.p.carry),
            Mos6502InstructionCode::Beq => bicondition!(self.registers.p.zero),
            Mos6502InstructionCode::Bmi => bicondition!(self.registers.p.negative),
            Mos6502InstructionCode::Bne => bicondition!(!self.registers.p.zero),
            Mos6502InstructionCode::Bpl => bicondition!(!self.registers.p.negative),
            Mos6502InstructionCode::Bvc => bicondition!(!self.registers.p.overflow),
            Mos6502InstructionCode::Bvs => bicondition!(self.registers.p.overflow),
            _ => panic!("This instruction doesn't have biconditional cycles."),
        }
    }
}

#[cfg(test)]
mod tests {
    use instruction::AddressingMode;
    use AVAILABLE_MEMORY;
    use Mos6502Cpu;

    #[test]
    fn it_shouldnt_access_zero_page_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        {
            let mem = cpu.get_memory_slice(0, 0x200);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0x20, 0x200);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0x20, 0x1FF);
            assert!(mem.is_err());
        }
    }

    #[test]
    fn it_shouldnt_access_stack_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        {
            let mem = cpu.get_memory_slice(0x100, 0x300);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFF, 0x1FA);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0x101, 0x1FA);
            assert!(mem.is_err());
        }
    }

    #[test]
    fn it_shouldnt_access_interrupt_handlers_memory() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        {
            let mem = cpu.get_memory_slice(0x100, 0xFFFB);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFFFA, 0x10001);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFFF5, 0x10001);
            assert!(mem.is_err());
        }
        {
            let mem = cpu.get_memory_slice(0xFFFB, 0xFFFC);
            assert!(mem.is_err());
        }
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_accumulator() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0x42;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::Accumulator).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_immediate() {
        let cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::Immediate { byte: 0x42 }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x35] = 0x42;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::ZeroPage { byte: 0x35 }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x20] = 0x42;
        cpu.registers.x = 0x60;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::ZeroPageIndexedX { byte: 0xC0 }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x20] = 0x42;
        cpu.registers.y = 0x60;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::ZeroPageIndexedY { byte: 0xC0 }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x2442] = 0x42;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x42,
            }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x2443] = 0x42;
        cpu.registers.x = 1;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::AbsoluteIndexedX {
                high_byte: 0x24,
                low_byte: 0x42,
            }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x2443] = 0x42;
        cpu.registers.y = 1;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::AbsoluteIndexedY {
                high_byte: 0x24,
                low_byte: 0x42,
            }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_indexed_indirect() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x24] = 0x74;
        cpu.memory[0x25] = 0x20;
        cpu.memory[0x2074] = 0x42;
        cpu.registers.x = 0x04;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::IndexedIndirect {
                byte: 0x20,
            }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_indirect_indexed() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x86] = 0x28;
        cpu.memory[0x87] = 0x40;
        cpu.memory[0x4028] = 0x42;
        cpu.registers.y = 0x10;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::IndexedIndirect {
                byte: 0x86,
            }).unwrap(),
            0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_accumulator() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.a = 0;
        cpu.set_value_to_addressing_mode(&AddressingMode::Accumulator, 0x42)
            .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_zero_page() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.set_value_to_addressing_mode(
            &AddressingMode::ZeroPage { byte: 0x35 }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x35], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_zero_page_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x60;
        cpu.set_value_to_addressing_mode(
            &AddressingMode::ZeroPageIndexedX { byte: 0xC0 }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x20], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_zero_page_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x60;
        cpu.set_value_to_addressing_mode(
            &AddressingMode::ZeroPageIndexedY { byte: 0xC0 }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x20], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_absolute() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.set_value_to_addressing_mode(&AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x42,
        }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x2442], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_absolute_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 1;
        cpu.set_value_to_addressing_mode(&AddressingMode::AbsoluteIndexedX {
            high_byte: 0x24,
            low_byte: 0x42,
        }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x2443], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_absolute_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 1;
        cpu.set_value_to_addressing_mode(&AddressingMode::AbsoluteIndexedY {
            high_byte: 0x24,
            low_byte: 0x42,
        }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x2443], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_indexed_indirect() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x24] = 0x74;
        cpu.memory[0x25] = 0x20;
        cpu.registers.x = 0x04;
        cpu.set_value_to_addressing_mode(&AddressingMode::IndexedIndirect {
            byte: 0x20,
        }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x2074], 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_indirect_indexed() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x86] = 0x28;
        cpu.memory[0x87] = 0x40;
        cpu.registers.y = 0x10;
        cpu.set_value_to_addressing_mode(&AddressingMode::IndexedIndirect {
            byte: 0x86,
        }, 0x42).unwrap();
        assert_eq!(cpu.memory[0x4028], 0x42);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_absolute() {
        let cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::Absolute {
            high_byte: 0x42,
            low_byte: 0x24,
        }).unwrap();
        assert_eq!(address, 0x4224);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_indirect() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x0120] = 0xFC;
        cpu.memory[0x0121] = 0xBA;
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::Indirect {
            high_byte: 0x01,
            low_byte: 0x20,
        }).unwrap();
        assert_eq!(address, 0xBAFC);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_zero_page() {
        let cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::ZeroPage {
            byte: 0x42,
        }).unwrap();
        assert_eq!(address, 0x42);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_zero_page_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x80;
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::ZeroPageIndexedX {
            byte: 0xff,
        }).unwrap();
        assert_eq!(address, 0x7f);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_absolute_indexed_by_x() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.x = 0x80;
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::AbsoluteIndexedX {
            low_byte: 0xff,
            high_byte: 0x00,
        }).unwrap();
        assert_eq!(address, 0x17f);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_absolute_indexed_by_y() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.registers.y = 0x80;
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::AbsoluteIndexedY {
            low_byte: 0xff,
            high_byte: 0x00,
        }).unwrap();
        assert_eq!(address, 0x17f);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_indexed_indirect() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x24] = 0x74;
        cpu.memory[0x25] = 0x20;
        cpu.registers.x = 0x04;
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::IndexedIndirect {
            byte: 0x20,
        }).unwrap();
        assert_eq!(address, 0x2074);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_indirect_indexed() {
        let mut cpu = Mos6502Cpu::new([0; AVAILABLE_MEMORY]);
        cpu.memory[0x86] = 0x28;
        cpu.memory[0x87] = 0x40;
        cpu.registers.y = 0x10;
        let address = cpu.get_address_from_addressing_mode(&AddressingMode::IndexedIndirect {
            byte: 0x86,
        }).unwrap();
        assert_eq!(address, 0x4028);
    }
}