use cpu::{RegisterType, Location, Address};

pub(crate) enum Cycles {
    Single(u8),
    Conditional { not_met: u8, met: u8 },
}

#[derive(Clone)]
pub enum Instruction {
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
    Sta { address: Address},
    Lda { address: Address},
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
    Jnz { address: Address},
    Jmp { address: Address},
    Cnz { address: Address},
    Push { register: RegisterType },
    Adi { byte: u8 },
    Rst { value: u8 },
    Rz,
    Ret,
    Jz { address: Address},
    Cz { address: Address},
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

impl Instruction {
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Instruction {
        match bytes[0] {
            0x00 => Instruction::Noop,
            0x01 => Instruction::Lxi { register: RegisterType::B, low_byte: bytes[1], high_byte: bytes[2] },
            0x02 => Instruction::Stax { register: RegisterType::B },
            0x03 => Instruction::Inx { register: RegisterType::B },
            0x04 => Instruction::Inr { source: Location::Register { register: RegisterType::B } },
            0x05 => Instruction::Dcr { source: Location::Register { register: RegisterType::B } },
            0x06 => Instruction::Mvi { source: Location::Register { register: RegisterType::B }, byte: bytes[1] },
            0x07 => Instruction::Rlc,
            0x09 => Instruction::Dad { register: RegisterType::B },
            0x0a => Instruction::Ldax { register: RegisterType::B },
            0x0b => Instruction::Dcx { register: RegisterType::B },
            0x0c => Instruction::Inr { source: Location::Register { register: RegisterType::C } },
            0x0d => Instruction::Dcr { source: Location::Register { register: RegisterType::C } },
            0x0e => Instruction::Mvi { source: Location::Register { register: RegisterType::C }, byte: bytes[1] },
            0x0f => Instruction::Rrc,
            0x11 => Instruction::Lxi { register: RegisterType::D, low_byte: bytes[1], high_byte: bytes[2] },
            0x12 => Instruction::Stax { register: RegisterType::D },
            0x13 => Instruction::Inx { register: RegisterType::D },
            0x14 => Instruction::Inr { source: Location::Register { register: RegisterType::D } },
            0x15 => Instruction::Dcr { source: Location::Register { register: RegisterType::D } },
            0x16 => Instruction::Mvi { source: Location::Register { register: RegisterType::D }, byte: bytes[1] },
            0x17 => Instruction::Ral,
            0x19 => Instruction::Dad { register: RegisterType::D },
            0x1a => Instruction::Ldax { register: RegisterType::D },
            0x1b => Instruction::Dcx { register: RegisterType::D },
            0x1c => Instruction::Inr { source: Location::Register { register: RegisterType::E } },
            0x1d => Instruction::Dcr { source: Location::Register { register: RegisterType::E } },
            0x1e => Instruction::Mvi { source: Location::Register { register: RegisterType::E }, byte: bytes[1] },
            0x1f => Instruction::Rar,
            0x21 => Instruction::Lxi { register: RegisterType::H, low_byte: bytes[1], high_byte: bytes[2] },
            0x22 => Instruction::Shld { address: [bytes[1], bytes[2]] },
            0x23 => Instruction::Inx { register: RegisterType::H },
            0x24 => Instruction::Inr { source: Location::Register { register: RegisterType::H } },
            0x25 => Instruction::Dcr { source: Location::Register { register: RegisterType::H } },
            0x26 => Instruction::Mvi { source: Location::Register { register: RegisterType::H }, byte: bytes[1] },
            0x27 => Instruction::Daa,
            0x29 => Instruction::Dad { register: RegisterType::H },
            0x2a => Instruction::Lhld { address: [bytes[1], bytes[2]] },
            0x2b => Instruction::Dcx { register: RegisterType::H },
            0x2c => Instruction::Inr { source: Location::Register { register: RegisterType::L } },
            0x2d => Instruction::Dcr { source: Location::Register { register: RegisterType::L } },
            0x2e => Instruction::Mvi { source: Location::Register { register: RegisterType::L }, byte: bytes[1] },
            0x2f => Instruction::Cma,
            0x31 => Instruction::Lxi { register: RegisterType::Sp, low_byte: bytes[1], high_byte: bytes[2] },
            0x32 => Instruction::Sta { address: [bytes[1], bytes[2]] },
            0x33 => Instruction::Inx { register: RegisterType::Sp },
            0x34 => Instruction::Inr { source: Location::Memory },
            0x35 => Instruction::Dcr { source: Location::Memory },
            0x36 => Instruction::Mvi { source: Location::Memory, byte: bytes[1] },
            0x37 => Instruction::Stc,
            0x39 => Instruction::Dad { register: RegisterType::Sp },
            0x3a => Instruction::Lda { address: [bytes[1], bytes[2]] },
            0x3b => Instruction::Dcx { register: RegisterType::Sp },
            0x3c => Instruction::Inr { source: Location::Register { register: RegisterType::A } },
            0x3d => Instruction::Dcr { source: Location::Register { register: RegisterType::A } },
            0x3e => Instruction::Mvi { source: Location::Register { register: RegisterType::A }, byte: bytes[1] },
            0x3f => Instruction::Cmc,
            0x40 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::B }
                },
            0x41 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::C }
                },
            0x42 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::D }
                },
            0x43 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::E }
                },
            0x44 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::H }
                },
            0x45 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::L }
                },
            0x46 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Memory,
                },
            0x47 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::B },
                    source: Location::Register { register: RegisterType::A }
                },
            0x48 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::B }
                },
            0x49 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::C }
                },
            0x4a =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::D }
                },
            0x4b =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::E }
                },
            0x4c =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::H }
                },
            0x4d =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::L }
                },
            0x4e =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Memory,
                },
            0x4f =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::C },
                    source: Location::Register { register: RegisterType::A }
                },
            0x50 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::B }
                },
            0x51 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::C }
                },
            0x52 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::D }
                },
            0x53 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::E }
                },
            0x54 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::H }
                },
            0x55 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::L }
                },
            0x56 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Memory,
                },
            0x57 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::D },
                    source: Location::Register { register: RegisterType::A }
                },
            0x58 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::B }
                },
            0x59 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::C }
                },
            0x5a =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::D }
                },
            0x5b =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::E }
                },
            0x5c =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::H }
                },
            0x5d =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::L }
                },
            0x5e =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Memory,
                },
            0x5f =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::E },
                    source: Location::Register { register: RegisterType::A }
                },
            0x60 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::B }
                },
            0x61 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::C }
                },
            0x62 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::D }
                },
            0x63 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::E }
                },
            0x64 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::H }
                },
            0x65 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::L }
                },
            0x66 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Memory,
                },
            0x67 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::H },
                    source: Location::Register { register: RegisterType::A }
                },
            0x68 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::B }
                },
            0x69 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::C }
                },
            0x6a =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::D }
                },
            0x6b =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::E }
                },
            0x6c =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::H }
                },
            0x6d =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::L }
                },
            0x6e =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Memory,
                },
            0x6f =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::L },
                    source: Location::Register { register: RegisterType::A }
                },
            0x70 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::B }
                },
            0x71 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::C }
                },
            0x72 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::D }
                },
            0x73 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::E }
                },
            0x74 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::H }
                },
            0x75 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::L }
                },
            0x76 =>
                Instruction::Hlt,
            0x77 =>
                Instruction::Mov {
                    destiny: Location::Memory,
                    source: Location::Register { register: RegisterType::A }
                },
            0x78 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::B }
                },
            0x79 =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::C }
                },
            0x7a =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::D }
                },
            0x7b =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::E }
                },
            0x7c =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::H }
                },
            0x7d =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::L }
                },
            0x7e =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Memory,
                },
            0x7f =>
                Instruction::Mov {
                    destiny: Location::Register { register: RegisterType::A },
                    source: Location::Register { register: RegisterType::A }
                },
            0x80 => Instruction::Add { source: Location::Register { register: RegisterType::B } },
            0x81 => Instruction::Add { source: Location::Register { register: RegisterType::C } },
            0x82 => Instruction::Add { source: Location::Register { register: RegisterType::D } },
            0x83 => Instruction::Add { source: Location::Register { register: RegisterType::E } },
            0x84 => Instruction::Add { source: Location::Register { register: RegisterType::H } },
            0x85 => Instruction::Add { source: Location::Register { register: RegisterType::L } },
            0x86 => Instruction::Add { source: Location::Memory },
            0x87 => Instruction::Add { source: Location::Register { register: RegisterType::A } },
            0x88 => Instruction::Adc { source: Location::Register { register: RegisterType::B } },
            0x89 => Instruction::Adc { source: Location::Register { register: RegisterType::C } },
            0x8a => Instruction::Adc { source: Location::Register { register: RegisterType::D } },
            0x8b => Instruction::Adc { source: Location::Register { register: RegisterType::E } },
            0x8c => Instruction::Adc { source: Location::Register { register: RegisterType::H } },
            0x8d => Instruction::Adc { source: Location::Register { register: RegisterType::L } },
            0x8e => Instruction::Adc { source: Location::Memory },
            0x8f => Instruction::Adc { source: Location::Register { register: RegisterType::A } },
            0x90 => Instruction::Sub { source: Location::Register { register: RegisterType::B } },
            0x91 => Instruction::Sub { source: Location::Register { register: RegisterType::C } },
            0x92 => Instruction::Sub { source: Location::Register { register: RegisterType::D } },
            0x93 => Instruction::Sub { source: Location::Register { register: RegisterType::E } },
            0x94 => Instruction::Sub { source: Location::Register { register: RegisterType::H } },
            0x95 => Instruction::Sub { source: Location::Register { register: RegisterType::L } },
            0x96 => Instruction::Sub { source: Location::Memory },
            0x97 => Instruction::Sub { source: Location::Register { register: RegisterType::A } },
            0x98 => Instruction::Sbb { source: Location::Register { register: RegisterType::B } },
            0x99 => Instruction::Sbb { source: Location::Register { register: RegisterType::C } },
            0x9a => Instruction::Sbb { source: Location::Register { register: RegisterType::D } },
            0x9b => Instruction::Sbb { source: Location::Register { register: RegisterType::E } },
            0x9c => Instruction::Sbb { source: Location::Register { register: RegisterType::H } },
            0x9d => Instruction::Sbb { source: Location::Register { register: RegisterType::L } },
            0x9e => Instruction::Sbb { source: Location::Memory },
            0x9f => Instruction::Sbb { source: Location::Register { register: RegisterType::A } },
            0xa0 => Instruction::Ana { source: Location::Register { register: RegisterType::B } },
            0xa1 => Instruction::Ana { source: Location::Register { register: RegisterType::C } },
            0xa2 => Instruction::Ana { source: Location::Register { register: RegisterType::D } },
            0xa3 => Instruction::Ana { source: Location::Register { register: RegisterType::E } },
            0xa4 => Instruction::Ana { source: Location::Register { register: RegisterType::H } },
            0xa5 => Instruction::Ana { source: Location::Register { register: RegisterType::L } },
            0xa6 => Instruction::Ana { source: Location::Memory },
            0xa7 => Instruction::Ana { source: Location::Register { register: RegisterType::A } },
            0xa8 => Instruction::Xra { source: Location::Register { register: RegisterType::B } },
            0xa9 => Instruction::Xra { source: Location::Register { register: RegisterType::C } },
            0xaa => Instruction::Xra { source: Location::Register { register: RegisterType::D } },
            0xab => Instruction::Xra { source: Location::Register { register: RegisterType::E } },
            0xac => Instruction::Xra { source: Location::Register { register: RegisterType::H } },
            0xad => Instruction::Xra { source: Location::Register { register: RegisterType::L } },
            0xae => Instruction::Xra { source: Location::Memory },
            0xaf => Instruction::Xra { source: Location::Register { register: RegisterType::A } },
            0xb0 => Instruction::Ora { source: Location::Register { register: RegisterType::B } },
            0xb1 => Instruction::Ora { source: Location::Register { register: RegisterType::C } },
            0xb2 => Instruction::Ora { source: Location::Register { register: RegisterType::D } },
            0xb3 => Instruction::Ora { source: Location::Register { register: RegisterType::E } },
            0xb4 => Instruction::Ora { source: Location::Register { register: RegisterType::H } },
            0xb5 => Instruction::Ora { source: Location::Register { register: RegisterType::L } },
            0xb6 => Instruction::Ora { source: Location::Memory },
            0xb7 => Instruction::Ora { source: Location::Register { register: RegisterType::A } },
            0xb8 => Instruction::Cmp { source: Location::Register { register: RegisterType::B } },
            0xb9 => Instruction::Cmp { source: Location::Register { register: RegisterType::C } },
            0xba => Instruction::Cmp { source: Location::Register { register: RegisterType::D } },
            0xbb => Instruction::Cmp { source: Location::Register { register: RegisterType::E } },
            0xbc => Instruction::Cmp { source: Location::Register { register: RegisterType::H } },
            0xbd => Instruction::Cmp { source: Location::Register { register: RegisterType::L } },
            0xbe => Instruction::Cmp { source: Location::Memory },
            0xbf => Instruction::Cmp { source: Location::Register { register: RegisterType::A } },
            0xc0 => Instruction::Rnz,
            0xc1 => Instruction::Pop { register: RegisterType::B },
            0xc2 => Instruction::Jnz { address: [bytes[1], bytes[2]] },
            0xc3 => Instruction::Jmp { address: [bytes[1], bytes[2]] },
            0xc4 => Instruction::Cnz { address: [bytes[1], bytes[2]] },
            0xc5 => Instruction::Push { register: RegisterType::B },
            0xc6 => Instruction::Adi { byte: bytes[1] },
            0xc7 => Instruction::Rst { value: 1 },
            0xc8 => Instruction::Rz,
            0xc9 => Instruction::Ret,
            0xca => Instruction::Jz { address: [bytes[1], bytes[2]] },
            0xcc => Instruction::Cz { address: [bytes[1], bytes[2]] },
            0xcd => Instruction::Call { address: [bytes[1], bytes[2]] },
            0xce => Instruction::Aci { byte: bytes[1] },
            0xcf => Instruction::Rst { value: 1 },
            0xd0 => Instruction::Rnc,
            0xd1 => Instruction::Pop { register: RegisterType::D },
            0xd2 => Instruction::Jnc { address: [bytes[1], bytes[2]] },
            0xd3 => Instruction::Out { byte: bytes[1] },
            0xd4 => Instruction::Cnc { address: [bytes[1], bytes[2]] },
            0xd5 => Instruction::Push { register: RegisterType::D },
            0xd6 => Instruction::Sui { byte: bytes[1] },
            0xd7 => Instruction::Rst { value: 2 },
            0xd8 => Instruction::Rc,
            0xda => Instruction::Jc { address: [bytes[1], bytes[2]] },
            0xdb => Instruction::In { byte: bytes[1] },
            0xdc => Instruction::Cc { address: [bytes[1], bytes[2]] },
            0xde => Instruction::Sbi { byte: bytes[1] },
            0xdf => Instruction::Rst { value: 3 },
            0xe0 => Instruction::Rpo,
            0xe1 => Instruction::Pop { register: RegisterType::H },
            0xe2 => Instruction::Jpo { address: [bytes[1], bytes[2]] },
            0xe3 => Instruction::Xthl,
            0xe4 => Instruction::Cpo { address: [bytes[1], bytes[2]] },
            0xe5 => Instruction::Push { register: RegisterType::H },
            0xe6 => Instruction::Ani { byte: bytes[1] },
            0xe7 => Instruction::Rst { value: 4 },
            0xe8 => Instruction::Rpe,
            0xe9 => Instruction::Pchl,
            0xea => Instruction::Jpe { address: [bytes[1], bytes[2]] },
            0xeb => Instruction::Xchg,
            0xec => Instruction::Cpe { address: [bytes[1], bytes[2]] },
            0xee => Instruction::Xri { byte: bytes[1] },
            0xef => Instruction::Rst { value: 5 },
            0xf0 => Instruction::Rp,
            0xf1 => Instruction::Pop { register: RegisterType::Psw },
            0xf2 => Instruction::Jp { address: [bytes[1], bytes[2]] },
            0xf3 => Instruction::Di,
            0xf4 => Instruction::Cp { address: [bytes[1], bytes[2]] },
            0xf5 => Instruction::Push { register: RegisterType::Psw },
            0xf6 => Instruction::Ori { byte: bytes[1] },
            0xf7 => Instruction::Rst { value: 6 },
            0xf8 => Instruction::Rm,
            0xf9 => Instruction::Sphl,
            0xfa => Instruction::Jm { address: [bytes[1], bytes[2]] },
            0xfb => Instruction::Ei,
            0xfc => Instruction::Cm { address: [bytes[1], bytes[2]] },
            0xfe => Instruction::Cpi { byte: bytes[1] },
            0xff => Instruction::Rst { value: 7 },
            c => {
                eprintln!("Unrecognized byte {}.", c);
                Instruction::Noop
            },
        }
    }

    pub fn size(&self) -> u8 {
        match self {
            Instruction::Noop => 1,
            Instruction::Lxi { register: _, low_byte: _, high_byte: _ } => 3,
            Instruction::Stax { register: _ } => 1,
            Instruction::Inx { register: _ } => 1,
            Instruction::Inr { source: _ } => 1,
            Instruction::Dcr { source: _ } => 1,
            Instruction::Mvi { source: _, byte: _ } => 2,
            Instruction::Rlc => 1,
            Instruction::Dad { register: _ } => 1,
            Instruction::Ldax { register: _ } => 1,
            Instruction::Dcx { register: _ } => 1,
            Instruction::Rrc => 1,
            Instruction::Ral => 1,
            Instruction::Rar => 1,
            Instruction::Shld { address: _ } => 3,
            Instruction::Daa => 1,
            Instruction::Lhld { address: _ } => 3,
            Instruction::Cma => 1,
            Instruction::Sta { address: _ } => 3,
            Instruction::Lda { address: _ } => 3,
            Instruction::Stc => 1,
            Instruction::Cmc => 1,
            Instruction::Mov { destiny: _, source: _ } => 1,
            Instruction::Hlt => 1,
            Instruction::Add { source: _ } => 1,
            Instruction::Adc { source: _ } => 1,
            Instruction::Sub { source: _ } => 1,
            Instruction::Sbb { source: _ } => 1,
            Instruction::Ana { source: _ } => 1,
            Instruction::Xra { source: _ } => 1,
            Instruction::Ora { source: _ } => 1,
            Instruction::Cmp { source: _ } => 1,
            Instruction::Rnz => 1,
            Instruction::Pop { register: _ } => 1,
            Instruction::Jnz { address: _ } => 3,
            Instruction::Jmp { address: _ } => 3,
            Instruction::Cnz { address: _ } => 3,
            Instruction::Push { register: _ } => 1,
            Instruction::Adi { byte: _ } => 2,
            Instruction::Rst { value: _ } => 1,
            Instruction::Rz => 1,
            Instruction::Ret => 1,
            Instruction::Jz { address: _ } => 3,
            Instruction::Cz { address: _ } => 3,
            Instruction::Call { address: _ } => 3,
            Instruction::Aci { byte: _ } => 2,
            Instruction::Rnc => 1,
            Instruction::Jnc { address: _ } => 3,
            Instruction::Out { byte: _ } => 2,
            Instruction::Cnc { address: _ } => 3,
            Instruction::Sui { byte: _ } => 2,
            Instruction::Rc => 1,
            Instruction::Jc { address: _ } => 3,
            Instruction::In { byte: _ } => 2,
            Instruction::Cc { address: _ } => 3,
            Instruction::Sbi { byte: _ } => 2,
            Instruction::Rpo => 1,
            Instruction::Jpo { address: _ } => 3,
            Instruction::Xthl => 1,
            Instruction::Cpo { address: _ } => 3,
            Instruction::Ani { byte: _ } => 2,
            Instruction::Rpe => 1,
            Instruction::Pchl => 1,
            Instruction::Jpe { address: _ } => 3,
            Instruction::Xchg => 1,
            Instruction::Cpe { address: _ } => 3,
            Instruction::Xri { byte: _ } => 2,
            Instruction::Rp => 1,
            Instruction::Jp { address: _ } => 3,
            Instruction::Di => 1,
            Instruction::Cp { address: _ } => 3,
            Instruction::Ori { byte: _ } => 2,
            Instruction::Rm => 1,
            Instruction::Sphl => 1,
            Instruction::Jm { address: _ } => 3,
            Instruction::Ei => 1,
            Instruction::Cm { address: _ } => 3,
            Instruction::Cpi { byte: _ } => 2,
        }
    }

    pub(crate) fn get_cycles(&self) -> Cycles {
        match self {
            Instruction::Noop => Cycles::Single(4),
            Instruction::Lxi { register: _, low_byte: _, high_byte: _ } => Cycles::Single(10),
            Instruction::Stax { register: _ } => Cycles::Single(7),
            Instruction::Inx { register: _ } => Cycles::Single(5),
            Instruction::Inr {
                source: Location::Register { register: _ }
            } => Cycles::Single(5),
            Instruction::Inr { source: _ } => Cycles::Single(10),
            Instruction::Dcr {
                source: Location::Register { register: _ }
            } => Cycles::Single(5),
            Instruction::Dcr { source: _ } => Cycles::Single(10),
            Instruction::Mvi { source: Location::Register { register: _ }, byte: _ } =>
                Cycles::Single(7),
            Instruction::Mvi { source: _, byte: _ } => Cycles::Single(10),
            Instruction::Rlc => Cycles::Single(4),
            Instruction::Dad { register: _ } => Cycles::Single(10),
            Instruction::Ldax { register: _ } => Cycles::Single(7),
            Instruction::Dcx { register: _ } => Cycles::Single(5),
            Instruction::Rrc => Cycles::Single(4),
            Instruction::Ral => Cycles::Single(4),
            Instruction::Rar => Cycles::Single(4),
            Instruction::Shld { address: _ } => Cycles::Single(16),
            Instruction::Daa => Cycles::Single(4),
            Instruction::Lhld { address: _ } => Cycles::Single(16),
            Instruction::Cma => Cycles::Single(4),
            Instruction::Sta { address: _ } => Cycles::Single(13),
            Instruction::Lda { address: _ } => Cycles::Single(13),
            Instruction::Stc => Cycles::Single(4),
            Instruction::Cmc => Cycles::Single(4),
            Instruction::Mov {
                destiny: Location::Register { register: _ },
                source: Location::Register { register: _ },
            } => Cycles::Single(5),
            Instruction::Mov { destiny: _, source: _ } => Cycles::Single(7),
            Instruction::Hlt => Cycles::Single(7),
            Instruction::Add {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Add { source: _ } => Cycles::Single(7),
            Instruction::Adc {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Adc { source: _ } => Cycles::Single(7),
            Instruction::Sub {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Sub { source: _ } => Cycles::Single(7),
            Instruction::Sbb {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Sbb { source: _ } => Cycles::Single(7),
            Instruction::Ana {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Ana { source: _ } => Cycles::Single(7),
            Instruction::Xra {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Xra { source: _ } => Cycles::Single(7),
            Instruction::Ora {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Ora { source: _ } => Cycles::Single(7),
            Instruction::Cmp {
                source: Location::Register { register: _ }
            } => Cycles::Single(4),
            Instruction::Cmp { source: _ } => Cycles::Single(7),
            Instruction::Rnz => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Pop { register: _ } => Cycles::Single(10),
            Instruction::Jnz { address: _ } => Cycles::Single(10),
            Instruction::Jmp { address: _ } => Cycles::Single(10),
            Instruction::Cnz { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Push { register: _ } => Cycles::Single(11),
            Instruction::Adi { byte: _ } => Cycles::Single(7),
            Instruction::Rst { value: _ } => Cycles::Single(11),
            Instruction::Rz => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Ret => Cycles::Single(10),
            Instruction::Jz { address: _ } => Cycles::Single(10),
            Instruction::Cz { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Call { address: _ } => Cycles::Single(17),
            Instruction::Aci { byte: _ } => Cycles::Single(7),
            Instruction::Rnc => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Jnc { address: _ } => Cycles::Single(10),
            Instruction::Out { byte: _ } => Cycles::Single(10),
            Instruction::Cnc { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Sui { byte: _ } => Cycles::Single(7),
            Instruction::Rc => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Jc { address: _ } => Cycles::Single(10),
            Instruction::In { byte: _ } => Cycles::Single(10),
            Instruction::Cc { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Sbi { byte: _ } => Cycles::Single(7),
            Instruction::Rpo => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Jpo { address: _ } => Cycles::Single(10),
            Instruction::Xthl => Cycles::Single(18),
            Instruction::Cpo { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Ani { byte: _ } => Cycles::Single(7),
            Instruction::Rpe => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Pchl => Cycles::Single(5),
            Instruction::Jpe { address: _ } => Cycles::Single(10),
            Instruction::Xchg => Cycles::Single(4),
            Instruction::Cpe { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Xri { byte: _ } => Cycles::Single(7),
            Instruction::Rp => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Jp { address: _ } => Cycles::Single(10),
            Instruction::Di => Cycles::Single(4),
            Instruction::Cp { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Ori { byte: _ } => Cycles::Single(7),
            Instruction::Rm => Cycles::Conditional { not_met: 5, met: 11 },
            Instruction::Sphl => Cycles::Single(5),
            Instruction::Jm { address: _ } => Cycles::Single(10),
            Instruction::Ei => Cycles::Single(4),
            Instruction::Cm { address: _ } => Cycles::Conditional { not_met: 11, met: 17 },
            Instruction::Cpi { byte: _ } => Cycles::Single(7),
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        match self {
            Instruction::Noop => String::from("NOOP"),
            Instruction::Lxi { register, low_byte, high_byte } =>
                format!("LXI {},#${:02x}{:02x}", register.to_string(), high_byte, low_byte),
            Instruction::Stax { register } => format!("STAX {}", register.to_string()),
            Instruction::Inx { register } => format!("INX {}", register.to_string()),
            Instruction::Inr { source } => format!("INR {}", source.to_string()),
            Instruction::Dcr { source } => format!("DCR {}", source.to_string()),
            Instruction::Mvi { source, byte } =>
                format!("MVI {},#${:02x}", source.to_string(), byte),
            Instruction::Rlc => String::from("RLC"),
            Instruction::Dad { register } => format!("DAD {}", register.to_string()),
            Instruction::Ldax { register } => format!("LDAX {}", register.to_string()),
            Instruction::Dcx { register } => format!("DCX {}", register.to_string()),
            Instruction::Rrc => String::from("RRC"),
            Instruction::Ral => String::from("RAL"),
            Instruction::Rar => String::from("RAR"),
            Instruction::Shld { address } =>
                format!("SHLD ${:02x}{:02x}", address[1], address[0]),
            Instruction::Daa => String::from("DAA"),
            Instruction::Lhld { address } =>
                format!("LHLD ${:02x}{:02x}", address[1], address[0]),
            Instruction::Cma => String::from("CMA"),
            Instruction::Sta { address } =>
                format!("STA ${:02x}{:02x}", address[1], address[0]),
            Instruction::Lda { address } =>
                format!("LDA ${:02x}{:02x}", address[1], address[0]),
            Instruction::Stc => String::from("STC"),
            Instruction::Cmc => String::from("SMC"),
            Instruction::Mov { destiny, source } =>
                format!("MOV {},{}", destiny.to_string(), source.to_string()),
            Instruction::Hlt => format!("HLT"),
            Instruction::Add { source } => format!("ADD {}", source.to_string()),
            Instruction::Adc { source } => format!("ADC {}", source.to_string()),
            Instruction::Sub { source } => format!("SUB {}", source.to_string()),
            Instruction::Sbb { source } => format!("SBB {}", source.to_string()),
            Instruction::Ana { source } => format!("ANA {}", source.to_string()),
            Instruction::Xra { source } => format!("XRA {}", source.to_string()),
            Instruction::Ora { source } => format!("ORA {}", source.to_string()),
            Instruction::Cmp { source } => format!("CMP {}", source.to_string()),
            Instruction::Rnz => String::from("RNZ"),
            Instruction::Pop { register } => format!("POP {}", register.to_string()),
            Instruction::Jnz { address } =>
                format!("JNZ ${:02x}{:02x}", address[1], address[0]),
            Instruction::Jmp { address } =>
                format!("JMP ${:02x}{:02x}", address[1], address[0]),
            Instruction::Cnz { address } =>
                format!("CNZ ${:02x}{:02x}", address[1], address[0]),
            Instruction::Push { register } => format!("PUSH {}", register.to_string()),
            Instruction::Adi { byte } => format!("ADI #${:02x}", byte),
            Instruction::Rst { value } => format!("RST {}", value),
            Instruction::Rz => String::from("RZ"),
            Instruction::Ret => String::from("RET"),
            Instruction::Jz { address } =>
                format!("JZ ${:02x}{:02x}", address[1], address[0]),
            Instruction::Cz { address } =>
                format!("CZ ${:02x}{:02x}", address[1], address[0]),
            Instruction::Call { address } =>
                format!("CALL ${:02x}{:02x}", address[1], address[0]),
            Instruction::Aci { byte } => format!("ACI #${:02x}", byte),
            Instruction::Rnc => String::from("RNC"),
            Instruction::Jnc { address } =>
                format!("JNC ${:02x}{:02x}", address[1], address[0]),
            Instruction::Out { byte } => format!("OUT #${:02x}", byte),
            Instruction::Cnc { address } =>
                format!("CNC ${:02x}{:02x}", address[1], address[0]),
            Instruction::Sui { byte } => format!("SUI #${:02x}", byte),
            Instruction::Rc => String::from("RC"),
            Instruction::Jc { address } =>
                format!("JC ${:02x}{:02x}", address[1], address[0]),
            Instruction::In { byte } => format!("IN #${:02x}", byte),
            Instruction::Cc { address } =>
                format!("CC ${:02x}{:02x}", address[1], address[0]),
            Instruction::Sbi { byte } => format!("SBI #${:02x}", byte),
            Instruction::Rpo => String::from("RPO"),
            Instruction::Jpo { address } =>
                format!("JPO ${:02x}{:02x}", address[1], address[0]),
            Instruction::Xthl => String::from("XTHL"),
            Instruction::Cpo { address } =>
                format!("CPO ${:02x}{:02x}", address[1], address[0]),
            Instruction::Ani { byte } => format!("ANI #${:02x}", byte),
            Instruction::Rpe => String::from("RPE"),
            Instruction::Pchl => String::from("PCHL"),
            Instruction::Jpe { address } =>
                format!("JPE ${:02x}{:02x}", address[1], address[0]),
            Instruction::Xchg => String::from("RNC"),
            Instruction::Cpe { address } =>
                format!("CPE ${:02x}{:02x}", address[1], address[0]),
            Instruction::Xri { byte } => format!("XRI #${:02x}", byte),
            Instruction::Rp => String::from("RP"),
            Instruction::Jp { address } =>
                format!("JP ${:02x}{:02x}", address[1], address[0]),
            Instruction::Di => String::from("DI"),
            Instruction::Cp { address } =>
                format!("CP ${:02x}{:02x}", address[1], address[0]),
            Instruction::Ori { byte } => format!("ORI #${:02x}", byte),
            Instruction::Rm => String::from("RN"),
            Instruction::Sphl => String::from("SPHL"),
            Instruction::Jm { address } =>
                format!("JM ${:02x}{:02x}", address[1], address[0]),
            Instruction::Ei => String::from("EI"),
            Instruction::Cm { address } =>
                format!("CM ${:02x}{:02x}", address[1], address[0]),
            Instruction::Cpi { byte } => format!("CPI #${:02x}", byte),
        }
    }
}