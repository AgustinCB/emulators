use super::instruction::{AddressingMode, Mos6502InstructionCode};
use bit_utils::two_bytes_to_word;
use cpu::{Cpu, Cycles, Instruction};
use failure::Error;
use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;
use {CpuResult, Mos6502Instruction};

pub const AVAILABLE_MEMORY: usize = 0x10000;
pub(crate) const INTERRUPT_HANDLERS_START: usize = 0xFFFA;

#[derive(Debug, Fail)]
pub enum CpuError {
    #[fail(
        display = "Attempt to access reserved memory. 0x0000-0x0200 and 0xFFFA to 0x10000 are reserved."
    )]
    ReservedMemory,
    #[fail(display = "Attempt to use invalid addressing mode.")]
    InvalidAddressingMode,
    #[fail(display = "The instruction doesn't support that kind of cycle calculation.")]
    InvalidCyclesCalculation,
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
            break_flag: true,
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
            break_flag: true,
            decimal: (byte & 0x08) > 0,
            interrupt_disable: (byte & 0x04) > 0,
            zero: (byte & 0x02) > 0,
            carry: (byte & 0x01) > 0,
        }
    }

    pub(crate) fn to_byte(&self) -> u8 {
        ((self.negative as u8) << 7)
            | ((self.overflow as u8) << 6)
            | 0x20
            | 0x10
            | ((self.decimal as u8) << 3)
            | ((self.interrupt_disable as u8) << 2)
            | ((self.zero as u8) << 1)
            | (self.carry as u8)
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

pub trait Memory {
    fn set(&mut self, index: u16, new_value: u8);
    fn get(&self, index: u16) -> u8;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Memory for [u8; AVAILABLE_MEMORY] {
    fn set(&mut self, index: u16, new_value: u8) {
        self[index as usize] = new_value;
    }
    fn get(&self, index: u16) -> u8 {
        self[index as usize]
    }
    fn len(&self) -> usize {
        AVAILABLE_MEMORY
    }
}

impl<T: Memory> Memory for Rc<RefCell<T>> {
    fn set(&mut self, index: u16, new_value: u8) {
        self.borrow_mut().set(index, new_value);
    }
    fn get(&self, index: u16) -> u8 {
        self.borrow().get(index)
    }
    fn len(&self) -> usize {
        self.borrow().len()
    }
}

pub struct Mos6502Cpu {
    pub(crate) memory: Box<Memory>,
    pub(crate) registers: RegisterSet,
    pub(crate) page_crossed: bool,
    pub(crate) decimal_enabled: bool,
}

impl Mos6502Cpu {
    pub fn new(memory: Box<Memory>) -> Mos6502Cpu {
        Mos6502Cpu {
            decimal_enabled: true,
            memory,
            registers: RegisterSet::new(),
            page_crossed: false,
        }
    }

    pub fn without_decimal(memory: Box<Memory>) -> Mos6502Cpu {
        Mos6502Cpu {
            decimal_enabled: false,
            memory,
            registers: RegisterSet::new(),
            page_crossed: false,
        }
    }

    #[inline]
    fn execute_nop(&self) {}

    #[inline]
    pub fn set_pc(&mut self, address: u16) {
        self.registers.pc = address;
    }

    pub(crate) fn get_address_from_addressing_mode(
        &self,
        addressing_mode: &AddressingMode,
    ) -> Result<u16, CpuError> {
        match addressing_mode {
            AddressingMode::Indirect {
                high_byte,
                low_byte,
            } => {
                let indirect_address = two_bytes_to_word(*high_byte, *low_byte);
                let (low_byte, high_byte) = (
                    self.memory.get(indirect_address),
                    self.memory.get(indirect_address + 1),
                );
                Ok(two_bytes_to_word(high_byte, low_byte))
            }
            AddressingMode::Absolute {
                high_byte,
                low_byte,
            } => Ok(two_bytes_to_word(*high_byte, *low_byte)),
            AddressingMode::AbsoluteIndexedX {
                low_byte,
                high_byte,
            } => {
                let address = two_bytes_to_word(*high_byte, *low_byte) as u16;
                Ok(address + u16::from(self.registers.x))
            }
            AddressingMode::AbsoluteIndexedY {
                low_byte,
                high_byte,
            } => {
                let address = two_bytes_to_word(*high_byte, *low_byte) as u16;
                Ok(address + u16::from(self.registers.y))
            }
            AddressingMode::ZeroPage { byte } => Ok(u16::from(*byte)),
            AddressingMode::ZeroPageIndexedX { byte } => {
                Ok(u16::from(self.registers.x.wrapping_add(*byte)))
            }
            AddressingMode::ZeroPageIndexedY { byte } => {
                Ok(u16::from(self.registers.y.wrapping_add(*byte)))
            }
            AddressingMode::IndexedIndirect { byte } => {
                let indirect_address = u16::from((u16::from(*byte) + u16::from(self.registers.x)) as u8);
                let (low_byte, high_byte) = (
                    self.memory.get(indirect_address),
                    self.memory.get(indirect_address + 1),
                );
                Ok(two_bytes_to_word(high_byte, low_byte))
            }
            AddressingMode::IndirectIndexed { byte } => {
                let (low_byte, high_byte) = (
                    self.memory.get(u16::from(*byte)),
                    self.memory.get(u16::from(*byte) + 1),
                );
                Ok(two_bytes_to_word(high_byte, low_byte) + u16::from(self.registers.y))
            }
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn get_value_from_addressing_mode(
        &self,
        addressing_mode: &AddressingMode,
    ) -> Result<u8, CpuError> {
        match addressing_mode {
            AddressingMode::Accumulator => Ok(self.registers.a),
            AddressingMode::Immediate { byte } => Ok(*byte),
            AddressingMode::ZeroPage { byte } => Ok(self.memory.get(u16::from(*byte))),
            AddressingMode::ZeroPageIndexedX { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            AddressingMode::ZeroPageIndexedY { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            AddressingMode::Absolute { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            AddressingMode::AbsoluteIndexedX { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            AddressingMode::AbsoluteIndexedY { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            AddressingMode::IndexedIndirect { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            AddressingMode::IndirectIndexed { .. } => Ok(self
                .memory
                .get(self.get_address_from_addressing_mode(addressing_mode)?)),
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    pub(crate) fn set_value_to_addressing_mode(
        &mut self,
        addressing_mode: &AddressingMode,
        new_value: u8,
    ) -> CpuResult {
        match addressing_mode {
            AddressingMode::Accumulator => {
                self.registers.a = new_value;
                Ok(())
            },
            AddressingMode::ZeroPage { byte } => {
                self.memory.set(u16::from(*byte), new_value);
                Ok(())
            },
            AddressingMode::ZeroPageIndexedX { byte } => {
                let address = u16::from(self.registers.x.wrapping_add(*byte));
                self.memory.set(address, new_value);
                Ok(())
            }
            AddressingMode::ZeroPageIndexedY { byte } => {
                let address = u16::from(self.registers.y.wrapping_add(*byte));
                self.memory.set(address, new_value);
                Ok(())
            }
            AddressingMode::Absolute {
                high_byte,
                low_byte,
            } => {
                let address = two_bytes_to_word(*high_byte, *low_byte);
                self.memory.set(address, new_value);
                Ok(())
            }
            AddressingMode::AbsoluteIndexedX {
                high_byte,
                low_byte,
            } => {
                let x = u16::from(self.registers.x);
                let address = two_bytes_to_word(*high_byte, *low_byte);
                self.memory.set(address + x, new_value);
                self.update_page_crossed_status(address, address + x);
                Ok(())
            }
            AddressingMode::AbsoluteIndexedY {
                high_byte,
                low_byte,
            } => {
                let y = u16::from(self.registers.y);
                let address = two_bytes_to_word(*high_byte, *low_byte);
                self.memory.set(address + y, new_value);
                self.update_page_crossed_status(address, address + y);
                Ok(())
            }
            AddressingMode::IndexedIndirect { byte } => {
                let indirect_address = u16::from((u16::from(*byte) + u16::from(self.registers.x)) as u8);
                let (low_byte, high_byte) = (
                    self.memory.get(indirect_address),
                    self.memory.get(indirect_address + 1),
                );
                self.memory
                    .set(two_bytes_to_word(high_byte, low_byte), new_value);
                Ok(())
            }
            AddressingMode::IndirectIndexed { byte } => {
                let y = u16::from(self.registers.y);
                let (low_byte, high_byte) = (
                    self.memory.get(u16::from(*byte)),
                    self.memory.get(u16::from(*byte) + 1),
                );
                let indirect_address = two_bytes_to_word(high_byte, low_byte);
                let direct_address = indirect_address + y;
                self.update_page_crossed_status(indirect_address, direct_address);
                self.memory.set(direct_address, new_value);
                Ok(())
            }
            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    #[inline]
    pub(crate) fn update_page_crossed_status(&mut self, original: u16, new: u16) {
        self.page_crossed = (original & 0xff00) == (new & 0xff00);
    }

    #[inline]
    pub(crate) fn push(&mut self, value: u8) {
        self.memory.set(u16::from(self.registers.s) + 0x100, value);
        self.registers.s = self.registers.s.wrapping_sub(1);
    }

    #[inline]
    pub(crate) fn pull(&mut self) -> u8 {
        self.registers.s = self.registers.s.wrapping_add(1);
        self.memory.get(u16::from(self.registers.s) + 0x100)
    }
}

impl Cpu<u8, Mos6502Instruction, CpuError> for Mos6502Cpu {
    fn get_cycles_for_instruction(
        &mut self,
        instruction: &Mos6502Instruction,
    ) -> Result<u8, Error> {
        let cycles = match instruction.get_cycles()? {
            Cycles::Single(cycles) => cycles,
            Cycles::OneCondition { not_met, met } => {
                self.get_cycles_from_one_condition(instruction, not_met, met)?
            }
            Cycles::TwoConditions {
                not_met,
                first_met,
                second_met,
            } => {
                self.get_cycles_from_two_conditions(instruction, not_met, first_met, second_met)?
            }
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
            Mos6502InstructionCode::Ahx => self.execute_ahx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Alr => self.execute_alr(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Anc => self.execute_anc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::And => self.execute_and(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Arr => self.execute_arr(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Asl => self.execute_asl(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Axs => self.execute_axs(&instruction.addressing_mode)?,
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
            Mos6502InstructionCode::Dcp => self.execute_dcp(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Dec => self.execute_dec(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Dex => self.execute_dex(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Dey => self.execute_dey(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Eor => self.execute_eor(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Inc => self.execute_inc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Inx => self.execute_inx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Iny => self.execute_iny(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Irq => self.execute_brk(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Isc => self.execute_isc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Jmp => self.execute_jmp(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Jsr => self.execute_jsr(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Las => self.execute_las(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Lax => self.execute_lax(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Lda => self.execute_lda(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Ldx => self.execute_ldx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Ldy => self.execute_ldy(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Lsr => self.execute_lsr(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Nmi => self.execute_nmi(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Nop => self.execute_nop(),
            Mos6502InstructionCode::Ora => self.execute_ora(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Pha => self.execute_pha(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Php => self.execute_php(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Pla => self.execute_pla(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Plp => self.execute_plp(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rla => self.execute_rla(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rol => self.execute_rol(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Ror => self.execute_ror(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rra => self.execute_rra(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rst => self.execute_rst(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rti => self.execute_rti(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Rts => self.execute_rts(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sax => self.execute_sax(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sbc => self.execute_sbc(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sec => self.execute_sec(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sed => self.execute_sed(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sei => self.execute_sei(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Shx => self.execute_shx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Shy => self.execute_shy(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Slo => self.execute_slo(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sre => self.execute_sre(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sta => self.execute_sta(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Stx => self.execute_stx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Sty => self.execute_sty(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tas => self.execute_tas(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tax => self.execute_tax(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tay => self.execute_tay(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tsx => self.execute_tsx(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Txa => self.execute_txa(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Txs => self.execute_txs(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Tya => self.execute_tya(&instruction.addressing_mode)?,
            Mos6502InstructionCode::Xaa => self.execute_xaa(&instruction.addressing_mode)?,
        };
        Ok(())
    }

    fn get_pc(&self) -> u16 {
        self.registers.pc
    }

    fn get_next_instruction_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(3);
        let from = self.registers.pc as usize;
        let to = min(from + 3, self.memory.len());
        for i in from..to {
            res.push(self.memory.get(i as u16));
        }
        res
    }

    fn can_run(&self, _: &Mos6502Instruction) -> bool {
        true
    }

    fn is_done(&self) -> bool {
        self.registers.pc as usize >= AVAILABLE_MEMORY
    }

    fn increase_pc(&mut self, steps: u8) {
        self.registers.pc += u16::from(steps)
    }

    fn get_cycles_from_one_condition(
        &self,
        instruction: &Mos6502Instruction,
        not_met: u8,
        met: u8,
    ) -> Result<u8, Error> {
        macro_rules! page_crossed_condition {
            () => {
                if self.page_crossed {
                    Ok(met)
                } else {
                    Ok(not_met)
                }
            };
        }
        match instruction.instruction {
            Mos6502InstructionCode::Adc => page_crossed_condition!(),
            Mos6502InstructionCode::And => page_crossed_condition!(),
            Mos6502InstructionCode::Cmp => page_crossed_condition!(),
            Mos6502InstructionCode::Eor => page_crossed_condition!(),
            Mos6502InstructionCode::Lax => page_crossed_condition!(),
            Mos6502InstructionCode::Lda => page_crossed_condition!(),
            Mos6502InstructionCode::Ldx => page_crossed_condition!(),
            Mos6502InstructionCode::Ldy => page_crossed_condition!(),
            Mos6502InstructionCode::Nop => page_crossed_condition!(),
            Mos6502InstructionCode::Ora => page_crossed_condition!(),
            Mos6502InstructionCode::Sbc => page_crossed_condition!(),
            _ => Err(Error::from(CpuError::InvalidCyclesCalculation)),
        }
    }

    fn get_cycles_from_two_conditions(
        &self,
        instruction: &Mos6502Instruction,
        not_met: u8,
        first_met: u8,
        second_met: u8,
    ) -> Result<u8, Error> {
        macro_rules! bicondition {
            ($condition:expr) => {
                if $condition {
                    if self.page_crossed {
                        Ok(second_met)
                    } else {
                        Ok(first_met)
                    }
                } else {
                    Ok(not_met)
                }
            };
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
            _ => Err(Error::from(CpuError::InvalidCyclesCalculation)),
        }
    }
}

#[cfg(test)]
mod tests {
    use instruction::AddressingMode;
    use mos6502cpu::{Mos6502Cpu, AVAILABLE_MEMORY};

    #[test]
    fn it_should_get_value_from_addressing_mode_for_accumulator() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0x42;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::Accumulator)
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_immediate() {
        let m = [0; AVAILABLE_MEMORY];
        let cpu = Mos6502Cpu::new(Box::new(m));
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::Immediate { byte: 0x42 })
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x35, 0x42);
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::ZeroPage { byte: 0x35 })
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page_indexed_by_x() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x20, 0x42);
        cpu.registers.x = 0x60;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::ZeroPageIndexedX { byte: 0xC0 })
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_zero_page_indexed_by_y() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x20, 0x42);
        cpu.registers.y = 0x60;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::ZeroPageIndexedY { byte: 0xC0 })
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x2442, 0x42);
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x42,
            })
            .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute_indexed_by_x() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x2443, 0x42);
        cpu.registers.x = 1;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::AbsoluteIndexedX {
                high_byte: 0x24,
                low_byte: 0x42,
            })
            .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_absolute_indexed_by_y() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x2443, 0x42);
        cpu.registers.y = 1;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::AbsoluteIndexedY {
                high_byte: 0x24,
                low_byte: 0x42,
            })
            .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_indexed_indirect() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x24, 0x74);
        cpu.memory.set(0x25, 0x20);
        cpu.memory.set(0x2074, 0x42);
        cpu.registers.x = 0x04;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::IndexedIndirect { byte: 0x20 })
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_get_value_from_addressing_mode_for_indirect_indexed() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x86, 0x28);
        cpu.memory.set(0x87, 0x40);
        cpu.memory.set(0x4028, 0x42);
        cpu.registers.y = 0x10;
        assert_eq!(
            cpu.get_value_from_addressing_mode(&AddressingMode::IndexedIndirect { byte: 0x86 })
                .unwrap(),
            0x42
        );
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_accumulator() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.a = 0;
        cpu.set_value_to_addressing_mode(&AddressingMode::Accumulator, 0x42)
            .unwrap();
        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_zero_page() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.set_value_to_addressing_mode(&AddressingMode::ZeroPage { byte: 0x35 }, 0x42)
            .unwrap();
        assert_eq!(cpu.memory.get(0x35), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_zero_page_indexed_by_x() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 0x60;
        cpu.set_value_to_addressing_mode(&AddressingMode::ZeroPageIndexedX { byte: 0xC0 }, 0x42)
            .unwrap();
        assert_eq!(cpu.memory.get(0x20), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_zero_page_indexed_by_y() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.y = 0x60;
        cpu.set_value_to_addressing_mode(&AddressingMode::ZeroPageIndexedY { byte: 0xC0 }, 0x42)
            .unwrap();
        assert_eq!(cpu.memory.get(0x20), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_absolute() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.set_value_to_addressing_mode(
            &AddressingMode::Absolute {
                high_byte: 0x24,
                low_byte: 0x42,
            },
            0x42,
        )
        .unwrap();
        assert_eq!(cpu.memory.get(0x2442), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_absolute_indexed_by_x() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 1;
        cpu.set_value_to_addressing_mode(
            &AddressingMode::AbsoluteIndexedX {
                high_byte: 0x24,
                low_byte: 0x42,
            },
            0x42,
        )
        .unwrap();
        assert_eq!(cpu.memory.get(0x2443), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_absolute_indexed_by_y() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.y = 1;
        cpu.set_value_to_addressing_mode(
            &AddressingMode::AbsoluteIndexedY {
                high_byte: 0x24,
                low_byte: 0x42,
            },
            0x42,
        )
        .unwrap();
        assert_eq!(cpu.memory.get(0x2443), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_indexed_indirect() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x24, 0x74);
        cpu.memory.set(0x25, 0x20);
        cpu.registers.x = 0x04;
        cpu.set_value_to_addressing_mode(&AddressingMode::IndexedIndirect { byte: 0x20 }, 0x42)
            .unwrap();
        assert_eq!(cpu.memory.get(0x2074), 0x42);
    }

    #[test]
    fn it_should_set_value_to_addressing_mode_for_indirect_indexed() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x86, 0x28);
        cpu.memory.set(0x87, 0x40);
        cpu.registers.y = 0x10;
        cpu.set_value_to_addressing_mode(&AddressingMode::IndexedIndirect { byte: 0x86 }, 0x42)
            .unwrap();
        assert_eq!(cpu.memory.get(0x4028), 0x42);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_absolute() {
        let m = [0; AVAILABLE_MEMORY];
        let cpu = Mos6502Cpu::new(Box::new(m));
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::Absolute {
                high_byte: 0x42,
                low_byte: 0x24,
            })
            .unwrap();
        assert_eq!(address, 0x4224);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_indirect() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x0120, 0xFC);
        cpu.memory.set(0x0121, 0xBA);
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::Indirect {
                high_byte: 0x01,
                low_byte: 0x20,
            })
            .unwrap();
        assert_eq!(address, 0xBAFC);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_zero_page() {
        let m = [0; AVAILABLE_MEMORY];
        let cpu = Mos6502Cpu::new(Box::new(m));
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::ZeroPage { byte: 0x42 })
            .unwrap();
        assert_eq!(address, 0x42);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_zero_page_indexed_by_x() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 0x80;
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::ZeroPageIndexedX { byte: 0xff })
            .unwrap();
        assert_eq!(address, 0x7f);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_absolute_indexed_by_x() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.x = 0x80;
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::AbsoluteIndexedX {
                low_byte: 0xff,
                high_byte: 0x00,
            })
            .unwrap();
        assert_eq!(address, 0x17f);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_absolute_indexed_by_y() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.registers.y = 0x80;
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::AbsoluteIndexedY {
                low_byte: 0xff,
                high_byte: 0x00,
            })
            .unwrap();
        assert_eq!(address, 0x17f);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_indexed_indirect() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x24, 0x74);
        cpu.memory.set(0x25, 0x20);
        cpu.registers.x = 0x04;
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::IndexedIndirect { byte: 0x20 })
            .unwrap();
        assert_eq!(address, 0x2074);
    }

    #[test]
    fn it_should_get_address_from_addressing_mode_for_indirect_indexed() {
        let m = [0; AVAILABLE_MEMORY];
        let mut cpu = Mos6502Cpu::new(Box::new(m));
        cpu.memory.set(0x86, 0x28);
        cpu.memory.set(0x87, 0x40);
        cpu.registers.y = 0x10;
        let address = cpu
            .get_address_from_addressing_mode(&AddressingMode::IndexedIndirect { byte: 0x86 })
            .unwrap();
        assert_eq!(address, 0x4028);
    }
}
