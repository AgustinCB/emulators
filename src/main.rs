use std::cmp::min;
use std::env::args;
use std::fs::File;
use std::io::Read;

#[derive(Clone)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    Sp,
    Psw,
}

type Address = [u8; 2];

#[derive(Clone)]
enum Instruction {
    Noop,
    Lxi { register: Register, low_byte: u8, high_byte: u8 },
    Stax { register: Register },
    Inx { register: Register },
    Inr { register: Register },
    Dcr { register: Register },
    Mvi { register: Register, byte: u8 },
    Rlc,
    Dad { register: Register },
    Ldax { register: Register },
    Dcx { register: Register },
    Rrc,
    Ral,
    Rar,
    Rim,
    Shld { address: Address },
    Daa,
    Lhld { address: Address },
    Cma,
    Sim,
    Sta { address: Address},
    Lda { address: Address},
    Stc,
    Cmc,
    Mov { source: Register, destiny: Register },
    Add { register: Register },
    Adc { register: Register },
    Sub { register: Register },
    Sbb { register: Register },
    Ana { register: Register },
    Xra { register: Register },
    Ora { register: Register },
    Cmp { register: Register },
    Rnz,
    Pop { register: Register },
    Jnz { address: Address},
    Jmp { address: Address},
    Cnz { address: Address},
    Push { register: Register },
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
    El,
    Cm { address: Address},
    Cpi { byte: u8 },
}

#[inline]
fn get_instruction(bytes: &[u8]) -> Result<(Instruction, u8), String> {
    match bytes[0] {
        0x00 => Ok((Instruction::Noop, 0)),
        0x01 => Ok((Instruction::Lxi { register: Register::B, low_byte: bytes[1], high_byte: bytes[2] }, 2)),
        0x02 => Ok((Instruction::Stax { register: Register::B }, 0)),
        0x03 => Ok((Instruction::Inx { register: Register::B }, 0)),
        0x04 => Ok((Instruction::Inr { register: Register::B }, 0)),
        0x05 => Ok((Instruction::Dcr { register: Register::B }, 0)),
        0x06 => Ok((Instruction::Mvi { register: Register::B, byte: bytes[1] }, 1)),
        0x07 => Ok((Instruction::Rlc, 0)),
        0x09 => Ok((Instruction::Dad { register: Register::B }, 0)),
        0x0a => Ok((Instruction::Ldax { register: Register::B }, 0)),
        0x0b => Ok((Instruction::Dcx { register: Register::B }, 0)),
        0x0c => Ok((Instruction::Inr { register: Register::C }, 0)),
        0x0d => Ok((Instruction::Dcr { register: Register::C }, 0)),
        0x0e => Ok((Instruction::Mvi { register: Register::C, byte: bytes[1] }, 1)),
        0x0f => Ok((Instruction::Rrc, 0)),
        0x11 => Ok((Instruction::Lxi { register: Register::D, low_byte: bytes[1], high_byte: bytes[2] }, 2)),
        0x12 => Ok((Instruction::Stax { register: Register::D }, 0)),
        0x13 => Ok((Instruction::Inx { register: Register::D }, 0)),
        0x14 => Ok((Instruction::Inr { register: Register::D }, 0)),
        0x15 => Ok((Instruction::Dcr { register: Register::D }, 0)),
        0x16 => Ok((Instruction::Mvi { register: Register::D, byte: bytes[1] }, 1)),
        0x17 => Ok((Instruction::Ral, 0)),
        0x19 => Ok((Instruction::Dad { register: Register::D }, 0)),
        0x1a => Ok((Instruction::Ldax { register: Register::D }, 0)),
        0x1b => Ok((Instruction::Dcx { register: Register::D }, 0)),
        0x1c => Ok((Instruction::Inr { register: Register::E }, 0)),
        0x1d => Ok((Instruction::Dcr { register: Register::E }, 0)),
        0x1e => Ok((Instruction::Mvi { register: Register::E, byte: bytes[1] }, 1)),
        0x1f => Ok((Instruction::Rar, 0)),
        0x20 => Ok((Instruction::Rim, 0)),
        0x21 => Ok((Instruction::Lxi { register: Register::H, low_byte: bytes[1], high_byte: bytes[2] }, 2)),
        0x22 => Ok((Instruction::Shld { address: [bytes[1], bytes[2]] }, 2)),
        0x23 => Ok((Instruction::Inx { register: Register::H }, 0)),
        0x24 => Ok((Instruction::Inr { register: Register::H }, 0)),
        0x25 => Ok((Instruction::Dcr { register: Register::H }, 0)),
        0x26 => Ok((Instruction::Mvi { register: Register::H, byte: bytes[1] }, 1)),
        0x27 => Ok((Instruction::Daa, 0)),
        0x29 => Ok((Instruction::Dad { register: Register::H }, 0)),
        0x2a => Ok((Instruction::Lhld { address: [bytes[1], bytes[2]] }, 2)),
        0x2b => Ok((Instruction::Dcx { register: Register::H }, 0)),
        0x2c => Ok((Instruction::Inr { register: Register::L }, 0)),
        0x2d => Ok((Instruction::Dcx { register: Register::L }, 0)),
        0x2e => Ok((Instruction::Mvi { register: Register::L, byte: bytes[1] }, 1)),
        0x2f => Ok((Instruction::Cma, 0)),
        0x30 => Ok((Instruction::Sim, 0)),
        0x31 => Ok((Instruction::Lxi { register: Register::Sp, low_byte: bytes[1], high_byte: bytes[2] }, 2)),
        0x32 => Ok((Instruction::Sta { address: [bytes[1], bytes[2]] }, 2)),
        0x33 => Ok((Instruction::Inx { register: Register::Sp }, 0)),
        0x34 => Ok((Instruction::Inr { register: Register::M }, 0)),
        0x35 => Ok((Instruction::Dcr { register: Register::M }, 0)),
        0x36 => Ok((Instruction::Mvi { register: Register::M, byte: bytes[1] }, 1)),
        0x37 => Ok((Instruction::Stc, 0)),
        0x39 => Ok((Instruction::Dad { register: Register::Sp }, 0)),
        0x3a => Ok((Instruction::Lda { address: [bytes[1], bytes[2]] }, 2)),
        0x3b => Ok((Instruction::Dcx { register: Register::Sp }, 0)),
        0x3c => Ok((Instruction::Inr { register: Register::A }, 0)),
        0x3d => Ok((Instruction::Dcx { register: Register::A }, 0)),
        0x3e => Ok((Instruction::Mvi { register: Register::A, byte: bytes[1] }, 1)),
        0x3f => Ok((Instruction::Cmc, 0)),
        0x40 => Ok((Instruction::Mov { source: Register::B, destiny: Register::B }, 0)),
        0x41 => Ok((Instruction::Mov { source: Register::B, destiny: Register::C }, 0)),
        0x42 => Ok((Instruction::Mov { source: Register::B, destiny: Register::D }, 0)),
        0x43 => Ok((Instruction::Mov { source: Register::B, destiny: Register::E }, 0)),
        0x44 => Ok((Instruction::Mov { source: Register::B, destiny: Register::H }, 0)),
        0x45 => Ok((Instruction::Mov { source: Register::B, destiny: Register::L }, 0)),
        0x46 => Ok((Instruction::Mov { source: Register::B, destiny: Register::M }, 0)),
        0x47 => Ok((Instruction::Mov { source: Register::B, destiny: Register::A }, 0)),
        0x48 => Ok((Instruction::Mov { source: Register::C, destiny: Register::B }, 0)),
        0x49 => Ok((Instruction::Mov { source: Register::C, destiny: Register::C }, 0)),
        0x4a => Ok((Instruction::Mov { source: Register::C, destiny: Register::D }, 0)),
        0x4b => Ok((Instruction::Mov { source: Register::C, destiny: Register::E }, 0)),
        0x4c => Ok((Instruction::Mov { source: Register::C, destiny: Register::H }, 0)),
        0x4d => Ok((Instruction::Mov { source: Register::C, destiny: Register::L }, 0)),
        0x4e => Ok((Instruction::Mov { source: Register::C, destiny: Register::M }, 0)),
        0x4f => Ok((Instruction::Mov { source: Register::C, destiny: Register::A }, 0)),
        0x50 => Ok((Instruction::Mov { source: Register::D, destiny: Register::B }, 0)),
        0x51 => Ok((Instruction::Mov { source: Register::D, destiny: Register::C }, 0)),
        0x52 => Ok((Instruction::Mov { source: Register::D, destiny: Register::D }, 0)),
        0x53 => Ok((Instruction::Mov { source: Register::D, destiny: Register::E }, 0)),
        0x54 => Ok((Instruction::Mov { source: Register::D, destiny: Register::H }, 0)),
        0x55 => Ok((Instruction::Mov { source: Register::D, destiny: Register::L }, 0)),
        0x56 => Ok((Instruction::Mov { source: Register::D, destiny: Register::M }, 0)),
        0x57 => Ok((Instruction::Mov { source: Register::D, destiny: Register::A }, 0)),
        0x58 => Ok((Instruction::Mov { source: Register::E, destiny: Register::B }, 0)),
        0x59 => Ok((Instruction::Mov { source: Register::E, destiny: Register::C }, 0)),
        0x5a => Ok((Instruction::Mov { source: Register::E, destiny: Register::D }, 0)),
        0x5b => Ok((Instruction::Mov { source: Register::E, destiny: Register::E }, 0)),
        0x5c => Ok((Instruction::Mov { source: Register::E, destiny: Register::H }, 0)),
        0x5d => Ok((Instruction::Mov { source: Register::E, destiny: Register::L }, 0)),
        0x5e => Ok((Instruction::Mov { source: Register::E, destiny: Register::M }, 0)),
        0x5f => Ok((Instruction::Mov { source: Register::E, destiny: Register::A }, 0)),
        0x60 => Ok((Instruction::Mov { source: Register::H, destiny: Register::B }, 0)),
        0x61 => Ok((Instruction::Mov { source: Register::H, destiny: Register::C }, 0)),
        0x62 => Ok((Instruction::Mov { source: Register::H, destiny: Register::D }, 0)),
        0x63 => Ok((Instruction::Mov { source: Register::H, destiny: Register::E }, 0)),
        0x64 => Ok((Instruction::Mov { source: Register::H, destiny: Register::H }, 0)),
        0x65 => Ok((Instruction::Mov { source: Register::H, destiny: Register::L }, 0)),
        0x66 => Ok((Instruction::Mov { source: Register::H, destiny: Register::M }, 0)),
        0x67 => Ok((Instruction::Mov { source: Register::H, destiny: Register::A }, 0)),
        0x68 => Ok((Instruction::Mov { source: Register::L, destiny: Register::B }, 0)),
        0x69 => Ok((Instruction::Mov { source: Register::L, destiny: Register::C }, 0)),
        0x6a => Ok((Instruction::Mov { source: Register::L, destiny: Register::D }, 0)),
        0x6b => Ok((Instruction::Mov { source: Register::L, destiny: Register::E }, 0)),
        0x6c => Ok((Instruction::Mov { source: Register::L, destiny: Register::H }, 0)),
        0x6d => Ok((Instruction::Mov { source: Register::L, destiny: Register::L }, 0)),
        0x6e => Ok((Instruction::Mov { source: Register::L, destiny: Register::M }, 0)),
        0x6f => Ok((Instruction::Mov { source: Register::L, destiny: Register::A }, 0)),
        0x70 => Ok((Instruction::Mov { source: Register::M, destiny: Register::B }, 0)),
        0x71 => Ok((Instruction::Mov { source: Register::M, destiny: Register::C }, 0)),
        0x72 => Ok((Instruction::Mov { source: Register::M, destiny: Register::D }, 0)),
        0x73 => Ok((Instruction::Mov { source: Register::M, destiny: Register::E }, 0)),
        0x74 => Ok((Instruction::Mov { source: Register::M, destiny: Register::H }, 0)),
        0x75 => Ok((Instruction::Mov { source: Register::M, destiny: Register::L }, 0)),
        0x76 => Ok((Instruction::Mov { source: Register::M, destiny: Register::M }, 0)),
        0x77 => Ok((Instruction::Mov { source: Register::M, destiny: Register::A }, 0)),
        0x78 => Ok((Instruction::Mov { source: Register::A, destiny: Register::B }, 0)),
        0x79 => Ok((Instruction::Mov { source: Register::A, destiny: Register::C }, 0)),
        0x7a => Ok((Instruction::Mov { source: Register::A, destiny: Register::D }, 0)),
        0x7b => Ok((Instruction::Mov { source: Register::A, destiny: Register::E }, 0)),
        0x7c => Ok((Instruction::Mov { source: Register::A, destiny: Register::H }, 0)),
        0x7d => Ok((Instruction::Mov { source: Register::A, destiny: Register::L }, 0)),
        0x7e => Ok((Instruction::Mov { source: Register::A, destiny: Register::M }, 0)),
        0x7f => Ok((Instruction::Mov { source: Register::A, destiny: Register::A }, 0)),
        0x80 => Ok((Instruction::Add { register: Register::B }, 0)),
        0x81 => Ok((Instruction::Add { register: Register::C }, 0)),
        0x82 => Ok((Instruction::Add { register: Register::D }, 0)),
        0x83 => Ok((Instruction::Add { register: Register::E }, 0)),
        0x84 => Ok((Instruction::Add { register: Register::H }, 0)),
        0x85 => Ok((Instruction::Add { register: Register::L }, 0)),
        0x86 => Ok((Instruction::Add { register: Register::M }, 0)),
        0x87 => Ok((Instruction::Add { register: Register::A }, 0)),
        0x88 => Ok((Instruction::Adc { register: Register::B }, 0)),
        0x89 => Ok((Instruction::Adc { register: Register::C }, 0)),
        0x8a => Ok((Instruction::Adc { register: Register::D }, 0)),
        0x8b => Ok((Instruction::Adc { register: Register::E }, 0)),
        0x8c => Ok((Instruction::Adc { register: Register::H }, 0)),
        0x8d => Ok((Instruction::Adc { register: Register::L }, 0)),
        0x8e => Ok((Instruction::Adc { register: Register::M }, 0)),
        0x8f => Ok((Instruction::Adc { register: Register::A }, 0)),
        0x90 => Ok((Instruction::Sub { register: Register::B }, 0)),
        0x91 => Ok((Instruction::Sub { register: Register::C }, 0)),
        0x92 => Ok((Instruction::Sub { register: Register::D }, 0)),
        0x93 => Ok((Instruction::Sub { register: Register::E }, 0)),
        0x94 => Ok((Instruction::Sub { register: Register::H }, 0)),
        0x95 => Ok((Instruction::Sub { register: Register::L }, 0)),
        0x96 => Ok((Instruction::Sub { register: Register::M }, 0)),
        0x97 => Ok((Instruction::Sub { register: Register::A }, 0)),
        0x98 => Ok((Instruction::Sbb { register: Register::B }, 0)),
        0x99 => Ok((Instruction::Sbb { register: Register::C }, 0)),
        0x9a => Ok((Instruction::Sbb { register: Register::D }, 0)),
        0x9b => Ok((Instruction::Sbb { register: Register::E }, 0)),
        0x9c => Ok((Instruction::Sbb { register: Register::H }, 0)),
        0x9d => Ok((Instruction::Sbb { register: Register::L }, 0)),
        0x9e => Ok((Instruction::Sbb { register: Register::M }, 0)),
        0x9f => Ok((Instruction::Sbb { register: Register::A }, 0)),
        0xa0 => Ok((Instruction::Ana { register: Register::B }, 0)),
        0xa1 => Ok((Instruction::Ana { register: Register::C }, 0)),
        0xa2 => Ok((Instruction::Ana { register: Register::D }, 0)),
        0xa3 => Ok((Instruction::Ana { register: Register::E }, 0)),
        0xa4 => Ok((Instruction::Ana { register: Register::H }, 0)),
        0xa5 => Ok((Instruction::Ana { register: Register::L }, 0)),
        0xa6 => Ok((Instruction::Ana { register: Register::M }, 0)),
        0xa7 => Ok((Instruction::Ana { register: Register::A }, 0)),
        0xa8 => Ok((Instruction::Xra { register: Register::B }, 0)),
        0xa9 => Ok((Instruction::Xra { register: Register::C }, 0)),
        0xaa => Ok((Instruction::Xra { register: Register::D }, 0)),
        0xab => Ok((Instruction::Xra { register: Register::E }, 0)),
        0xac => Ok((Instruction::Xra { register: Register::H }, 0)),
        0xad => Ok((Instruction::Xra { register: Register::L }, 0)),
        0xae => Ok((Instruction::Xra { register: Register::M }, 0)),
        0xaf => Ok((Instruction::Xra { register: Register::A }, 0)),
        0xb0 => Ok((Instruction::Ora { register: Register::B }, 0)),
        0xb1 => Ok((Instruction::Ora { register: Register::C }, 0)),
        0xb2 => Ok((Instruction::Ora { register: Register::D }, 0)),
        0xb3 => Ok((Instruction::Ora { register: Register::E }, 0)),
        0xb4 => Ok((Instruction::Ora { register: Register::H }, 0)),
        0xb5 => Ok((Instruction::Ora { register: Register::L }, 0)),
        0xb6 => Ok((Instruction::Ora { register: Register::M }, 0)),
        0xb7 => Ok((Instruction::Ora { register: Register::A }, 0)),
        0xb8 => Ok((Instruction::Cmp { register: Register::B }, 0)),
        0xb9 => Ok((Instruction::Cmp { register: Register::C }, 0)),
        0xba => Ok((Instruction::Cmp { register: Register::D }, 0)),
        0xbb => Ok((Instruction::Cmp { register: Register::E }, 0)),
        0xbc => Ok((Instruction::Cmp { register: Register::H }, 0)),
        0xbd => Ok((Instruction::Cmp { register: Register::L }, 0)),
        0xbe => Ok((Instruction::Cmp { register: Register::M }, 0)),
        0xbf => Ok((Instruction::Cmp { register: Register::A }, 0)),
        0xc0 => Ok((Instruction::Rnz, 0)),
        0xc1 => Ok((Instruction::Pop { register: Register::B }, 0)),
        0xc2 => Ok((Instruction::Jnz { address: [bytes[1], bytes[2]] }, 2)),
        0xc3 => Ok((Instruction::Jmp { address: [bytes[1], bytes[2]] }, 2)),
        0xc4 => Ok((Instruction::Cnz { address: [bytes[1], bytes[2]] }, 2)),
        0xc5 => Ok((Instruction::Push { register: Register::B }, 0)),
        0xc6 => Ok((Instruction::Adi { byte: bytes[1] }, 1)),
        0xc7 => Ok((Instruction::Rst { value: 1 }, 0)),
        0xc8 => Ok((Instruction::Rz, 0)),
        0xc9 => Ok((Instruction::Ret, 0)),
        0xca => Ok((Instruction::Jz { address: [bytes[1], bytes[2]] }, 2)),
        0xcc => Ok((Instruction::Cz { address: [bytes[1], bytes[2]] }, 2)),
        0xcd => Ok((Instruction::Call { address: [bytes[1], bytes[2]] }, 2)),
        0xce => Ok((Instruction::Aci { byte: bytes[1] }, 1)),
        0xcf => Ok((Instruction::Rst { value: 1 }, 0)),
        0xd0 => Ok((Instruction::Rnc, 0)),
        0xd1 => Ok((Instruction::Pop { register: Register::D }, 0)),
        0xd2 => Ok((Instruction::Jnc { address: [bytes[1], bytes[2]] }, 2)),
        0xd3 => Ok((Instruction::Out { byte: bytes[1] }, 1)),
        0xd4 => Ok((Instruction::Cnc { address: [bytes[1], bytes[2]] }, 2)),
        0xd5 => Ok((Instruction::Push { register: Register::D }, 0)),
        0xd6 => Ok((Instruction::Sui { byte: bytes[1] }, 1)),
        0xd7 => Ok((Instruction::Rst { value: 2 }, 0)),
        0xd8 => Ok((Instruction::Rc, 0)),
        0xda => Ok((Instruction::Jc { address: [bytes[1], bytes[2]] }, 1)),
        0xdb => Ok((Instruction::In { byte: bytes[1] }, 1)),
        0xdc => Ok((Instruction::Cc { address: [bytes[1], bytes[2]] }, 1)),
        0xde => Ok((Instruction::Sbi { byte: bytes[1] }, 1)),
        0xdf => Ok((Instruction::Rst { value: 3 }, 0)),
        0xe0 => Ok((Instruction::Rpo, 0)),
        0xe1 => Ok((Instruction::Pop { register: Register::H }, 0)),
        0xe2 => Ok((Instruction::Jpo { address: [bytes[1], bytes[2]] }, 2)),
        0xe3 => Ok((Instruction::Xthl, 0)),
        0xe4 => Ok((Instruction::Cpo { address: [bytes[1], bytes[2]] }, 2)),
        0xe5 => Ok((Instruction::Push { register: Register::H }, 0)),
        0xe6 => Ok((Instruction::Ani { byte: bytes[1] }, 1)),
        0xe7 => Ok((Instruction::Rst { value: 4 }, 0)),
        0xe8 => Ok((Instruction::Rpe, 0)),
        0xe9 => Ok((Instruction::Pchl, 0)),
        0xea => Ok((Instruction::Jpe { address: [bytes[1], bytes[2]] }, 2)),
        0xeb => Ok((Instruction::Xchg, 0)),
        0xec => Ok((Instruction::Cpe { address: [bytes[1], bytes[2]] }, 2)),
        0xee => Ok((Instruction::Xri { byte: bytes[1] }, 1)),
        0xef => Ok((Instruction::Rst { value: 5 }, 0)),
        0xf0 => Ok((Instruction::Rp, 0)),
        0xf1 => Ok((Instruction::Pop { register: Register::Psw }, 0)),
        0xf2 => Ok((Instruction::Jp { address: [bytes[1], bytes[2]] }, 2)),
        0xf3 => Ok((Instruction::Di, 0)),
        0xf4 => Ok((Instruction::Cp { address: [bytes[1], bytes[2]] }, 2)),
        0xf5 => Ok((Instruction::Push { register: Register::Psw }, 0)),
        0xf6 => Ok((Instruction::Ori { byte: bytes[1] }, 1)),
        0xf7 => Ok((Instruction::Rst { value: 6 }, 0)),
        0xf8 => Ok((Instruction::Rm, 0)),
        0xf9 => Ok((Instruction::Sphl, 0)),
        0xfa => Ok((Instruction::Jm { address: [bytes[1], bytes[2]] }, 2)),
        0xfb => Ok((Instruction::El, 0)),
        0xfc => Ok((Instruction::Cm { address: [bytes[1], bytes[2]] }, 2)),
        0xfe => Ok((Instruction::Cpi { byte: bytes[1] }, 1)),
        0xff => Ok((Instruction::Rst { value: 7 }, 0)),
        c => {
            eprint!("Unrecognized byte {}.\n", c);
            Ok((Instruction::Noop, 0))
        },
    }
}

fn get_instructions(bytes: Vec<u8>) -> Result<Vec<Instruction>, String> {
    let mut result = vec![Instruction::Noop; bytes.len()];
    let mut pass = 0;
    for index in 0..bytes.len() {
        if pass == 0 {
            let (i, extra_bytes) = get_instruction(&bytes[index..min(index+3, bytes.len())])?;
            result.push(i);
            pass = extra_bytes;
        } else {
            pass -= 1;
        }
    }
    Ok(result)
}

fn read_file(file_name: &str) -> std::io::Result<Vec<u8>> {
    let metadata = std::fs::metadata(file_name)?;
    let mut f = File::open(file_name)?;
    // this may blow up memory if the file is big enough
    // TODO: streams???
    let mut bytes = vec![0; metadata.len() as usize];
    f.read(&mut bytes[..])?;
    Ok(bytes)
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!("Usage: disassembler-8080 [file]")
    }
    let bytes = read_file(&args[1]).unwrap();
    let instructions = get_instructions(bytes).unwrap();
}
