use super::cpu::{Cycles, Instruction};
use super::failure::Error;
use intel8080cpu::{RegisterType, Location, Address};

#[derive(Debug, Fail)]
#[fail(display = "Instruction parsing error")]
pub struct Intel8080InstructionError {}

#[derive(Clone)]
pub enum Intel8080Instruction {
    Noop,
    Lxi { register: RegisterType, low_byte: u8, high_byte: u8 },
    Stax { register: RegisterType },
    Inx { register: RegisterType },
    Inr { source: Location },
    Dcr { source: Location },
    Mvi { source: Location, byte: u8 },
    Rlc,
    Dad { register: RegisterType },
    Ldax { register: RegisterType },
    Dcx { register: RegisterType },
    Rrc,
    Ral,
    Rar,
    Shld { address: Address },
    Daa,
    Lhld { address: Address },
    Cma,
    Sta { address: Address },
    Lda { address: Address },
    Stc,
    Cmc,
    Mov { destiny: Location, source: Location },
    Hlt,
    Add { source: Location },
    Adc { source: Location },
    Sub { source: Location },
    Sbb { source: Location },
    Ana { source: Location },
    Xra { source: Location },
    Ora { source: Location },
    Cmp { source: Location },
    Rnz,
    Pop { register: RegisterType },
    Jnz { address: Address },
    Jmp { address: Address },
    Cnz { address: Address },
    Push { register: RegisterType },
    Adi { byte: u8 },
    Rst { byte: u8 },
    Rz,
    Ret,
    Jz { address: Address },
    Cz { address: Address },
    Call { address: Address},
    Aci { byte: u8 },
    Rnc,
    Jnc { address: Address},
    Out { byte: u8 },
    Cnc { address: Address},
    Sui { byte: u8 },
    Rc,
    Jc { address: Address},
    In { byte: u8 },
    Cc { address: Address},
    Sbi { byte: u8 },
    Rpo,
    Jpo { address: Address},
    Xthl,
    Cpo { address: Address},
    Ani { byte: u8 },
    Rpe,
    Pchl,
    Jpe { address: Address},
    Xchg,
    Cpe { address: Address},
    Xri { byte: u8 },
    Rp,
    Jp { address: Address},
    Di,
    Cp { address: Address},
    Ori { byte: u8 },
    Rm,
    Sphl,
    Jm { address: Address},
    Ei,
    Cm { address: Address},
    Cpi { byte: u8 },
}

impl Instruction for Intel8080Instruction {
    fn size(&self) -> Result<u8, Error> {
        Ok(match self {
            Intel8080Instruction::Noop => 1,
            Intel8080Instruction::Lxi { register: _, low_byte: _, high_byte: _ } => 3,
            Intel8080Instruction::Stax { register: _ } => 1,
            Intel8080Instruction::Inx { register: _ } => 1,
            Intel8080Instruction::Inr { source: _ } => 1,
            Intel8080Instruction::Dcr { source: _ } => 1,
            Intel8080Instruction::Mvi { source: _, byte: _ } => 2,
            Intel8080Instruction::Rlc => 1,
            Intel8080Instruction::Dad { register: _ } => 1,
            Intel8080Instruction::Ldax { register: _ } => 1,
            Intel8080Instruction::Dcx { register: _ } => 1,
            Intel8080Instruction::Rrc => 1,
            Intel8080Instruction::Ral => 1,
            Intel8080Instruction::Rar => 1,
            Intel8080Instruction::Shld { address: _ } => 3,
            Intel8080Instruction::Daa => 1,
            Intel8080Instruction::Lhld { address: _ } => 3,
            Intel8080Instruction::Cma => 1,
            Intel8080Instruction::Sta { address: _ } => 3,
            Intel8080Instruction::Lda { address: _ } => 3,
            Intel8080Instruction::Stc => 1,
            Intel8080Instruction::Cmc => 1,
            Intel8080Instruction::Mov { destiny: _, source: _ } => 1,
            Intel8080Instruction::Hlt => 1,
            Intel8080Instruction::Add { source: _ } => 1,
            Intel8080Instruction::Adc { source: _ } => 1,
            Intel8080Instruction::Sub { source: _ } => 1,
            Intel8080Instruction::Sbb { source: _ } => 1,
            Intel8080Instruction::Ana { source: _ } => 1,
            Intel8080Instruction::Xra { source: _ } => 1,
            Intel8080Instruction::Ora { source: _ } => 1,
            Intel8080Instruction::Cmp { source: _ } => 1,
            Intel8080Instruction::Rnz => 1,
            Intel8080Instruction::Pop { register: _ } => 1,
            Intel8080Instruction::Jnz { address: _ } => 3,
            Intel8080Instruction::Jmp { address: _ } => 3,
            Intel8080Instruction::Cnz { address: _ } => 3,
            Intel8080Instruction::Push { register: _ } => 1,
            Intel8080Instruction::Adi { byte: _ } => 2,
            Intel8080Instruction::Rst { byte: _ } => 1,
            Intel8080Instruction::Rz => 1,
            Intel8080Instruction::Ret => 1,
            Intel8080Instruction::Jz { address: _ } => 3,
            Intel8080Instruction::Cz { address: _ } => 3,
            Intel8080Instruction::Call { address: _ } => 3,
            Intel8080Instruction::Aci { byte: _ } => 2,
            Intel8080Instruction::Rnc => 1,
            Intel8080Instruction::Jnc { address: _ } => 3,
            Intel8080Instruction::Out { byte: _ } => 2,
            Intel8080Instruction::Cnc { address: _ } => 3,
            Intel8080Instruction::Sui { byte: _ } => 2,
            Intel8080Instruction::Rc => 1,
            Intel8080Instruction::Jc { address: _ } => 3,
            Intel8080Instruction::In { byte: _ } => 2,
            Intel8080Instruction::Cc { address: _ } => 3,
            Intel8080Instruction::Sbi { byte: _ } => 2,
            Intel8080Instruction::Rpo => 1,
            Intel8080Instruction::Jpo { address: _ } => 3,
            Intel8080Instruction::Xthl => 1,
            Intel8080Instruction::Cpo { address: _ } => 3,
            Intel8080Instruction::Ani { byte: _ } => 2,
            Intel8080Instruction::Rpe => 1,
            Intel8080Instruction::Pchl => 1,
            Intel8080Instruction::Jpe { address: _ } => 3,
            Intel8080Instruction::Xchg => 1,
            Intel8080Instruction::Cpe { address: _ } => 3,
            Intel8080Instruction::Xri { byte: _ } => 2,
            Intel8080Instruction::Rp => 1,
            Intel8080Instruction::Jp { address: _ } => 3,
            Intel8080Instruction::Di => 1,
            Intel8080Instruction::Cp { address: _ } => 3,
            Intel8080Instruction::Ori { byte: _ } => 2,
            Intel8080Instruction::Rm => 1,
            Intel8080Instruction::Sphl => 1,
            Intel8080Instruction::Jm { address: _ } => 3,
            Intel8080Instruction::Ei => 1,
            Intel8080Instruction::Cm { address: _ } => 3,
            Intel8080Instruction::Cpi { byte: _ } => 2,
        })
    }

    fn get_cycles(&self) -> Result<Cycles, Error> {
        Ok(match self {
            Intel8080Instruction::Noop => single!(4),
            Intel8080Instruction::Lxi { register: _, low_byte: _, high_byte: _ } => single!(10),
            Intel8080Instruction::Stax { register: _ } => single!(7),
            Intel8080Instruction::Inx { register: _ } => single!(5),
            Intel8080Instruction::Inr {
                source: Location::Register { register: _ }
            } => single!(5),
            Intel8080Instruction::Inr { source: _ } => single!(10),
            Intel8080Instruction::Dcr {
                source: Location::Register { register: _ }
            } => single!(5),
            Intel8080Instruction::Dcr { source: _ } => single!(10),
            Intel8080Instruction::Mvi { source: Location::Register { register: _ }, byte: _ } =>
                single!(7),
            Intel8080Instruction::Mvi { source: _, byte: _ } => single!(10),
            Intel8080Instruction::Rlc => single!(4),
            Intel8080Instruction::Dad { register: _ } => single!(10),
            Intel8080Instruction::Ldax { register: _ } => single!(7),
            Intel8080Instruction::Dcx { register: _ } => single!(5),
            Intel8080Instruction::Rrc => single!(4),
            Intel8080Instruction::Ral => single!(4),
            Intel8080Instruction::Rar => single!(4),
            Intel8080Instruction::Shld { address: _ } => single!(16),
            Intel8080Instruction::Daa => single!(4),
            Intel8080Instruction::Lhld { address: _ } => single!(16),
            Intel8080Instruction::Cma => single!(4),
            Intel8080Instruction::Sta { address: _ } => single!(13),
            Intel8080Instruction::Lda { address: _ } => single!(13),
            Intel8080Instruction::Stc => single!(4),
            Intel8080Instruction::Cmc => single!(4),
            Intel8080Instruction::Mov {
                destiny: Location::Register { register: _ },
                source: Location::Register { register: _ },
            } => single!(5),
            Intel8080Instruction::Mov { destiny: _, source: _ } => single!(7),
            Intel8080Instruction::Hlt => single!(7),
            Intel8080Instruction::Add {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Add { source: _ } => single!(7),
            Intel8080Instruction::Adc {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Adc { source: _ } => single!(7),
            Intel8080Instruction::Sub {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Sub { source: _ } => single!(7),
            Intel8080Instruction::Sbb {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Sbb { source: _ } => single!(7),
            Intel8080Instruction::Ana {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Ana { source: _ } => single!(7),
            Intel8080Instruction::Xra {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Xra { source: _ } => single!(7),
            Intel8080Instruction::Ora {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Ora { source: _ } => single!(7),
            Intel8080Instruction::Cmp {
                source: Location::Register { register: _ }
            } => single!(4),
            Intel8080Instruction::Cmp { source: _ } => single!(7),
            Intel8080Instruction::Rnz => conditional!(5, 11),
            Intel8080Instruction::Pop { register: _ } => single!(10),
            Intel8080Instruction::Jnz { address: _ } => single!(10),
            Intel8080Instruction::Jmp { address: _ } => single!(10),
            Intel8080Instruction::Cnz { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Push { register: _ } => single!(11),
            Intel8080Instruction::Adi { byte: _ } => single!(7),
            Intel8080Instruction::Rst { byte: _ } => single!(11),
            Intel8080Instruction::Rz => conditional!(5, 11),
            Intel8080Instruction::Ret => single!(10),
            Intel8080Instruction::Jz { address: _ } => single!(10),
            Intel8080Instruction::Cz { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Call { address: _ } => single!(17),
            Intel8080Instruction::Aci { byte: _ } => single!(7),
            Intel8080Instruction::Rnc => conditional!(5, 11),
            Intel8080Instruction::Jnc { address: _ } => single!(10),
            Intel8080Instruction::Out { byte: _ } => single!(10),
            Intel8080Instruction::Cnc { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Sui { byte: _ } => single!(7),
            Intel8080Instruction::Rc => conditional!(5, 11),
            Intel8080Instruction::Jc { address: _ } => single!(10),
            Intel8080Instruction::In { byte: _ } => single!(10),
            Intel8080Instruction::Cc { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Sbi { byte: _ } => single!(7),
            Intel8080Instruction::Rpo => conditional!(5, 11),
            Intel8080Instruction::Jpo { address: _ } => single!(10),
            Intel8080Instruction::Xthl => single!(18),
            Intel8080Instruction::Cpo { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Ani { byte: _ } => single!(7),
            Intel8080Instruction::Rpe => conditional!(5, 11),
            Intel8080Instruction::Pchl => single!(5),
            Intel8080Instruction::Jpe { address: _ } => single!(10),
            Intel8080Instruction::Xchg => single!(4),
            Intel8080Instruction::Cpe { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Xri { byte: _ } => single!(7),
            Intel8080Instruction::Rp => conditional!(5, 11),
            Intel8080Instruction::Jp { address: _ } => single!(10),
            Intel8080Instruction::Di => single!(4),
            Intel8080Instruction::Cp { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Ori { byte: _ } => single!(7),
            Intel8080Instruction::Rm => conditional!(5, 11),
            Intel8080Instruction::Sphl => single!(5),
            Intel8080Instruction::Jm { address: _ } => single!(10),
            Intel8080Instruction::Ei => single!(4),
            Intel8080Instruction::Cm { address: _ } => conditional!(11, 17),
            Intel8080Instruction::Cpi { byte: _ } => single!(7),
        })
    }
}

impl From<Vec<u8>> for Intel8080Instruction {
    #[inline]
    fn from(bytes: Vec<u8>) -> Intel8080Instruction {
        match bytes[0] {
            0x00 => Intel8080Instruction::Noop,
            0x01 => Intel8080Instruction::Lxi { register: RegisterType::B, low_byte: bytes[1], high_byte: bytes[2] },
            0x02 => Intel8080Instruction::Stax { register: RegisterType::B },
            0x03 => Intel8080Instruction::Inx { register: RegisterType::B },
            0x04 => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::B } },
            0x05 => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::B } },
            0x06 => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::B }, byte: bytes[1] },
            0x07 => Intel8080Instruction::Rlc,
            0x09 => Intel8080Instruction::Dad { register: RegisterType::B },
            0x0a => Intel8080Instruction::Ldax { register: RegisterType::B },
            0x0b => Intel8080Instruction::Dcx { register: RegisterType::B },
            0x0c => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::C } },
            0x0d => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::C } },
            0x0e => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::C }, byte: bytes[1] },
            0x0f => Intel8080Instruction::Rrc,
            0x11 => Intel8080Instruction::Lxi { register: RegisterType::D, low_byte: bytes[1], high_byte: bytes[2] },
            0x12 => Intel8080Instruction::Stax { register: RegisterType::D },
            0x13 => Intel8080Instruction::Inx { register: RegisterType::D },
            0x14 => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::D } },
            0x15 => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::D } },
            0x16 => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::D }, byte: bytes[1] },
            0x17 => Intel8080Instruction::Ral,
            0x19 => Intel8080Instruction::Dad { register: RegisterType::D },
            0x1a => Intel8080Instruction::Ldax { register: RegisterType::D },
            0x1b => Intel8080Instruction::Dcx { register: RegisterType::D },
            0x1c => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::E } },
            0x1d => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::E } },
            0x1e => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::E }, byte: bytes[1] },
            0x1f => Intel8080Instruction::Rar,
            0x21 => Intel8080Instruction::Lxi { register: RegisterType::H, low_byte: bytes[1], high_byte: bytes[2] },
            0x22 => Intel8080Instruction::Shld { address: [bytes[1], bytes[2]] },
            0x23 => Intel8080Instruction::Inx { register: RegisterType::H },
            0x24 => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::H } },
            0x25 => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::H } },
            0x26 => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::H }, byte: bytes[1] },
            0x27 => Intel8080Instruction::Daa,
            0x29 => Intel8080Instruction::Dad { register: RegisterType::H },
            0x2a => Intel8080Instruction::Lhld { address: [bytes[1], bytes[2]] },
            0x2b => Intel8080Instruction::Dcx { register: RegisterType::H },
            0x2c => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::L } },
            0x2d => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::L } },
            0x2e => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::L }, byte: bytes[1] },
            0x2f => Intel8080Instruction::Cma,
            0x31 => Intel8080Instruction::Lxi { register: RegisterType::Sp, low_byte: bytes[1], high_byte: bytes[2] },
            0x32 => Intel8080Instruction::Sta { address: [bytes[1], bytes[2]] },
            0x33 => Intel8080Instruction::Inx { register: RegisterType::Sp },
            0x34 => Intel8080Instruction::Inr { source: Location::Memory },
            0x35 => Intel8080Instruction::Dcr { source: Location::Memory },
            0x36 => Intel8080Instruction::Mvi { source: Location::Memory, byte: bytes[1] },
            0x37 => Intel8080Instruction::Stc,
            0x39 => Intel8080Instruction::Dad { register: RegisterType::Sp },
            0x3a => Intel8080Instruction::Lda { address: [bytes[1], bytes[2]] },
            0x3b => Intel8080Instruction::Dcx { register: RegisterType::Sp },
            0x3c => Intel8080Instruction::Inr { source: Location::Register { register: RegisterType::A } },
            0x3d => Intel8080Instruction::Dcr { source: Location::Register { register: RegisterType::A } },
            0x3e => Intel8080Instruction::Mvi { source: Location::Register { register: RegisterType::A }, byte: bytes[1] },
            0x3f => Intel8080Instruction::Cmc,
            0x40 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::B }
                },
            0x41 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::C }
                },
            0x42 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::D }
                },
            0x43 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::E }
                },
            0x44 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::H }
                },
            0x45 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::L }
                },
            0x46 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Memory,
                },
            0x47 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::A }
                },
            0x48 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::B }
                },
            0x49 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::C }
                },
            0x4a =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::D }
                },
            0x4b =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::E }
                },
            0x4c =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::H }
                },
            0x4d =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::L }
                },
            0x4e =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Memory,
                },
            0x4f =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::A }
                },
            0x50 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::B }
                },
            0x51 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::C }
                },
            0x52 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::D }
                },
            0x53 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::E }
                },
            0x54 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::H }
                },
            0x55 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::L }
                },
            0x56 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Memory,
                },
            0x57 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::A }
                },
            0x58 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::B }
                },
            0x59 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::C }
                },
            0x5a =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::D }
                },
            0x5b =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::E }
                },
            0x5c =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::H }
                },
            0x5d =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::L }
                },
            0x5e =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Memory,
                },
            0x5f =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::A }
                },
            0x60 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::B }
                },
            0x61 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::C }
                },
            0x62 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::D }
                },
            0x63 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::E }
                },
            0x64 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::H }
                },
            0x65 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::L }
                },
            0x66 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Memory,
                },
            0x67 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::A }
                },
            0x68 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::B }
                },
            0x69 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::C }
                },
            0x6a =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::D }
                },
            0x6b =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::E }
                },
            0x6c =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::H }
                },
            0x6d =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::L }
                },
            0x6e =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Memory,
                },
            0x6f =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::A }
                },
            0x70 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::B }
                },
            0x71 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::C }
                },
            0x72 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::D }
                },
            0x73 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::E }
                },
            0x74 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::H }
                },
            0x75 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::L }
                },
            0x76 =>
                Intel8080Instruction::Hlt,
            0x77 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::A }
                },
            0x78 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::B }
                },
            0x79 =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::C }
                },
            0x7a =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::D }
                },
            0x7b =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::E }
                },
            0x7c =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::H }
                },
            0x7d =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::L }
                },
            0x7e =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Memory,
                },
            0x7f =>
                Intel8080Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::A }
                },
            0x80 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::B } },
            0x81 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::C } },
            0x82 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::D } },
            0x83 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::E } },
            0x84 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::H } },
            0x85 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::L } },
            0x86 => Intel8080Instruction::Add { source: Location::Memory },
            0x87 => Intel8080Instruction::Add { source: Location::Register { register: RegisterType::A } },
            0x88 => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::B } },
            0x89 => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::C } },
            0x8a => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::D } },
            0x8b => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::E } },
            0x8c => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::H } },
            0x8d => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::L } },
            0x8e => Intel8080Instruction::Adc { source: Location::Memory },
            0x8f => Intel8080Instruction::Adc { source: Location::Register { register: RegisterType::A } },
            0x90 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::B } },
            0x91 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::C } },
            0x92 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::D } },
            0x93 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::E } },
            0x94 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::H } },
            0x95 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::L } },
            0x96 => Intel8080Instruction::Sub { source: Location::Memory },
            0x97 => Intel8080Instruction::Sub { source: Location::Register { register: RegisterType::A } },
            0x98 => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::B } },
            0x99 => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::C } },
            0x9a => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::D } },
            0x9b => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::E } },
            0x9c => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::H } },
            0x9d => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::L } },
            0x9e => Intel8080Instruction::Sbb { source: Location::Memory },
            0x9f => Intel8080Instruction::Sbb { source: Location::Register { register: RegisterType::A } },
            0xa0 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::B } },
            0xa1 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::C } },
            0xa2 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::D } },
            0xa3 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::E } },
            0xa4 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::H } },
            0xa5 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::L } },
            0xa6 => Intel8080Instruction::Ana { source: Location::Memory },
            0xa7 => Intel8080Instruction::Ana { source: Location::Register { register: RegisterType::A } },
            0xa8 => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::B } },
            0xa9 => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::C } },
            0xaa => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::D } },
            0xab => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::E } },
            0xac => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::H } },
            0xad => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::L } },
            0xae => Intel8080Instruction::Xra { source: Location::Memory },
            0xaf => Intel8080Instruction::Xra { source: Location::Register { register: RegisterType::A } },
            0xb0 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::B } },
            0xb1 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::C } },
            0xb2 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::D } },
            0xb3 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::E } },
            0xb4 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::H } },
            0xb5 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::L } },
            0xb6 => Intel8080Instruction::Ora { source: Location::Memory },
            0xb7 => Intel8080Instruction::Ora { source: Location::Register { register: RegisterType::A } },
            0xb8 => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::B } },
            0xb9 => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::C } },
            0xba => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::D } },
            0xbb => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::E } },
            0xbc => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::H } },
            0xbd => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::L } },
            0xbe => Intel8080Instruction::Cmp { source: Location::Memory },
            0xbf => Intel8080Instruction::Cmp { source: Location::Register { register: RegisterType::A } },
            0xc0 => Intel8080Instruction::Rnz,
            0xc1 => Intel8080Instruction::Pop { register: RegisterType::B },
            0xc2 => Intel8080Instruction::Jnz { address: [bytes[1], bytes[2]] },
            0xc3 => Intel8080Instruction::Jmp { address: [bytes[1], bytes[2]] },
            0xc4 => Intel8080Instruction::Cnz { address: [bytes[1], bytes[2]] },
            0xc5 => Intel8080Instruction::Push { register: RegisterType::B },
            0xc6 => Intel8080Instruction::Adi { byte: bytes[1] },
            0xc7 => Intel8080Instruction::Rst { byte: 0 },
            0xc8 => Intel8080Instruction::Rz,
            0xc9 => Intel8080Instruction::Ret,
            0xca => Intel8080Instruction::Jz { address: [bytes[1], bytes[2]] },
            0xcc => Intel8080Instruction::Cz { address: [bytes[1], bytes[2]] },
            0xcd => Intel8080Instruction::Call { address: [bytes[1], bytes[2]] },
            0xce => Intel8080Instruction::Aci { byte: bytes[1] },
            0xcf => Intel8080Instruction::Rst { byte: 1 },
            0xd0 => Intel8080Instruction::Rnc,
            0xd1 => Intel8080Instruction::Pop { register: RegisterType::D },
            0xd2 => Intel8080Instruction::Jnc { address: [bytes[1], bytes[2]] },
            0xd3 => Intel8080Instruction::Out { byte: bytes[1] },
            0xd4 => Intel8080Instruction::Cnc { address: [bytes[1], bytes[2]] },
            0xd5 => Intel8080Instruction::Push { register: RegisterType::D },
            0xd6 => Intel8080Instruction::Sui { byte: bytes[1] },
            0xd7 => Intel8080Instruction::Rst { byte: 2 },
            0xd8 => Intel8080Instruction::Rc,
            0xda => Intel8080Instruction::Jc { address: [bytes[1], bytes[2]] },
            0xdb => Intel8080Instruction::In { byte: bytes[1] },
            0xdc => Intel8080Instruction::Cc { address: [bytes[1], bytes[2]] },
            0xde => Intel8080Instruction::Sbi { byte: bytes[1] },
            0xdf => Intel8080Instruction::Rst { byte: 3 },
            0xe0 => Intel8080Instruction::Rpo,
            0xe1 => Intel8080Instruction::Pop { register: RegisterType::H },
            0xe2 => Intel8080Instruction::Jpo { address: [bytes[1], bytes[2]] },
            0xe3 => Intel8080Instruction::Xthl,
            0xe4 => Intel8080Instruction::Cpo { address: [bytes[1], bytes[2]] },
            0xe5 => Intel8080Instruction::Push { register: RegisterType::H },
            0xe6 => Intel8080Instruction::Ani { byte: bytes[1] },
            0xe7 => Intel8080Instruction::Rst { byte: 4 },
            0xe8 => Intel8080Instruction::Rpe,
            0xe9 => Intel8080Instruction::Pchl,
            0xea => Intel8080Instruction::Jpe { address: [bytes[1], bytes[2]] },
            0xeb => Intel8080Instruction::Xchg,
            0xec => Intel8080Instruction::Cpe { address: [bytes[1], bytes[2]] },
            0xee => Intel8080Instruction::Xri { byte: bytes[1] },
            0xef => Intel8080Instruction::Rst { byte: 5 },
            0xf0 => Intel8080Instruction::Rp,
            0xf1 => Intel8080Instruction::Pop { register: RegisterType::Psw },
            0xf2 => Intel8080Instruction::Jp { address: [bytes[1], bytes[2]] },
            0xf3 => Intel8080Instruction::Di,
            0xf4 => Intel8080Instruction::Cp { address: [bytes[1], bytes[2]] },
            0xf5 => Intel8080Instruction::Push { register: RegisterType::Psw },
            0xf6 => Intel8080Instruction::Ori { byte: bytes[1] },
            0xf7 => Intel8080Instruction::Rst { byte: 6 },
            0xf8 => Intel8080Instruction::Rm,
            0xf9 => Intel8080Instruction::Sphl,
            0xfa => Intel8080Instruction::Jm { address: [bytes[1], bytes[2]] },
            0xfb => Intel8080Instruction::Ei,
            0xfc => Intel8080Instruction::Cm { address: [bytes[1], bytes[2]] },
            0xfe => Intel8080Instruction::Cpi { byte: bytes[1] },
            0xff => Intel8080Instruction::Rst { byte: 7 },
            c => {
                eprintln!("Unrecognized byte {}.", c);
                Intel8080Instruction::Noop
            },
        }
    }
}

impl ToString for Intel8080Instruction {
    fn to_string(&self) -> String {
        match self {
            Intel8080Instruction::Noop => String::from("NOP"),
            Intel8080Instruction::Lxi { register, low_byte, high_byte } =>
                format!("LXI {},#${:02x}{:02x}", register.to_string(), high_byte, low_byte),
            Intel8080Instruction::Stax { register } => format!("STAX {}", register.to_string()),
            Intel8080Instruction::Inx { register } => format!("INX {}", register.to_string()),
            Intel8080Instruction::Inr { source } => format!("INR {}", source.to_string()),
            Intel8080Instruction::Dcr { source } => format!("DCR {}", source.to_string()),
            Intel8080Instruction::Mvi { source, byte } =>
                format!("MVI {},#${:02x}", source.to_string(), byte),
            Intel8080Instruction::Rlc => String::from("RLC"),
            Intel8080Instruction::Dad { register } => format!("DAD {}", register.to_string()),
            Intel8080Instruction::Ldax { register } => format!("LDAX {}", register.to_string()),
            Intel8080Instruction::Dcx { register } => format!("DCX {}", register.to_string()),
            Intel8080Instruction::Rrc => String::from("RRC"),
            Intel8080Instruction::Ral => String::from("RAL"),
            Intel8080Instruction::Rar => String::from("RAR"),
            Intel8080Instruction::Shld { address } =>
                format!("SHLD ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Daa => String::from("DAA"),
            Intel8080Instruction::Lhld { address } =>
                format!("LHLD ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Cma => String::from("CMA"),
            Intel8080Instruction::Sta { address } =>
                format!("STA ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Lda { address } =>
                format!("LDA ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Stc => String::from("STC"),
            Intel8080Instruction::Cmc => String::from("SMC"),
            Intel8080Instruction::Mov { destiny, source } =>
                format!("MOV {},{}", destiny.to_string(), source.to_string()),
            Intel8080Instruction::Hlt => format!("HLT"),
            Intel8080Instruction::Add { source } => format!("ADD {}", source.to_string()),
            Intel8080Instruction::Adc { source } => format!("ADC {}", source.to_string()),
            Intel8080Instruction::Sub { source } => format!("SUB {}", source.to_string()),
            Intel8080Instruction::Sbb { source } => format!("SBB {}", source.to_string()),
            Intel8080Instruction::Ana { source } => format!("ANA {}", source.to_string()),
            Intel8080Instruction::Xra { source } => format!("XRA {}", source.to_string()),
            Intel8080Instruction::Ora { source } => format!("ORA {}", source.to_string()),
            Intel8080Instruction::Cmp { source } => format!("CMP {}", source.to_string()),
            Intel8080Instruction::Rnz => String::from("RNZ"),
            Intel8080Instruction::Pop { register } => format!("POP {}", register.to_string()),
            Intel8080Instruction::Jnz { address } =>
                format!("JNZ ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Jmp { address } =>
                format!("JMP ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Cnz { address } =>
                format!("CNZ ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Push { register } => format!("PUSH {}", register.to_string()),
            Intel8080Instruction::Adi { byte } => format!("ADI #${:02x}", byte),
            Intel8080Instruction::Rst { byte } => format!("RST {}", byte),
            Intel8080Instruction::Rz => String::from("RZ"),
            Intel8080Instruction::Ret => String::from("RET"),
            Intel8080Instruction::Jz { address } =>
                format!("JZ ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Cz { address } =>
                format!("CZ ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Call { address } =>
                format!("CALL ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Aci { byte } => format!("ACI #${:02x}", byte),
            Intel8080Instruction::Rnc => String::from("RNC"),
            Intel8080Instruction::Jnc { address } =>
                format!("JNC ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Out { byte } => format!("OUT #${:02x}", byte),
            Intel8080Instruction::Cnc { address } =>
                format!("CNC ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Sui { byte } => format!("SUI #${:02x}", byte),
            Intel8080Instruction::Rc => String::from("RC"),
            Intel8080Instruction::Jc { address } =>
                format!("JC ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::In { byte } => format!("IN #${:02x}", byte),
            Intel8080Instruction::Cc { address } =>
                format!("CC ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Sbi { byte } => format!("SBI #${:02x}", byte),
            Intel8080Instruction::Rpo => String::from("RPO"),
            Intel8080Instruction::Jpo { address } =>
                format!("JPO ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Xthl => String::from("XTHL"),
            Intel8080Instruction::Cpo { address } =>
                format!("CPO ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Ani { byte } => format!("ANI #${:02x}", byte),
            Intel8080Instruction::Rpe => String::from("RPE"),
            Intel8080Instruction::Pchl => String::from("PCHL"),
            Intel8080Instruction::Jpe { address } =>
                format!("JPE ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Xchg => String::from("RNC"),
            Intel8080Instruction::Cpe { address } =>
                format!("CPE ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Xri { byte } => format!("XRI #${:02x}", byte),
            Intel8080Instruction::Rp => String::from("RP"),
            Intel8080Instruction::Jp { address } =>
                format!("JP ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Di => String::from("DI"),
            Intel8080Instruction::Cp { address } =>
                format!("CP ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Ori { byte } => format!("ORI #${:02x}", byte),
            Intel8080Instruction::Rm => String::from("RM"),
            Intel8080Instruction::Sphl => String::from("SPHL"),
            Intel8080Instruction::Jm { address } =>
                format!("JM ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Ei => String::from("EI"),
            Intel8080Instruction::Cm { address } =>
                format!("CM ${:02x}{:02x}", address[1], address[0]),
            Intel8080Instruction::Cpi { byte } => format!("CPI #${:02x}", byte),
        }
    }
}