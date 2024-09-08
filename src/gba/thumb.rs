use std::{fmt::write, mem::offset_of};

#[derive(Debug, PartialEq)]
pub enum ThumbOpCode {
    ADC{rd: u8, rs: u8},
    ADD(InnerAdd),
    AND{rd: u8, rs: u8},
    ASR(InnerShift),
    B,
    BIC{rd: u8, rs: u8},
    BL,
    BX,
    CMN{rd: u8, rs: u8},
    CMP(InnerCmp),
    EOR{rd: u8, rs: u8},
    LDMIA,
    LDR,
    LDRB,
    LDRH,
    LSL(InnerShift),
    LDSB,
    LDSH,
    LSR(InnerShift),
    MOV(InnerMov),
    MUL{rd: u8, rs: u8},
    MVN{rd: u8, rs: u8},
    NEG{rd: u8, rs: u8},
    ORR{rd: u8, rs: u8},
    POP,
    PUSH,
    ROR{rd: u8, rs: u8},
    SBC{rd: u8, rs: u8},
    STMIAIA,
    STR,
    STRB,
    STRH,
    SWI,
    SUB(InnerSub),
    TST{rd: u8, rs: u8},
} 

#[derive(Debug, PartialEq)]
pub enum InnerAdd {
    AddReg{rd: u8, rs: u8, rn: u8},
    AddImm{rd: u8, rs: u8, offset: u8},
    AddByteImm{rd: u8, offset: u8},
    AddLowToHi{hd: u8, ss: u8},
    AddHiToLow{rd: u8, hs: u8},
    AddHiToHi{hd: u8, hs: u8},
}

#[derive(Debug, PartialEq)]
pub enum InnerSub {
    SubReg{rd: u8, rs: u8, rn: u8},
    SubImm{rd: u8, rs: u8, offset: u8},
    SubByteImm{rd: u8, offset: u8},
}

#[derive(Debug, PartialEq)]
pub enum InnerCmp {
    CmpByteImm{rd: u8, offset: u8},
    CmpAlu{rd: u8, rs: u8},
    CmpLowToHi{hd: u8, rs: u8},
    CmpHiToLow{rd: u8, hs: u8},
    CmpHiToHi{hd: u8, hs: u8}
}

#[derive(Debug, PartialEq)]
pub enum InnerMov{
    Offset{rd: u8, offset: u8},
    HiToLow{hs: u8, rd: u8},
    LowToHi{rs: u8, hd: u8},
    HiToHi{hs: u8, hd: u8},
}

#[derive(Debug, PartialEq)]
pub enum InnerBranchEx {
    Low{rs: u8},
    Hi{hs: u8},
}

#[derive(Debug, PartialEq)]
pub enum InnerShift {
    Alu{rs: u8, rd: u8},
    Reg{offset: u8, rs: u8, rd: u8}
}

fn get_triplet(value: u16, shift: u32) -> u8 {
    (value >> shift & 0x7) as u8
}

impl From<u16> for ThumbOpCode {
    fn from(value: u16) -> Self {
        if value & 0xf800 == 0x1800 {
            let i: u8 = (value >> 10 & 0x1) as u8;
            let op: u8 = (value >> 9 & 0x1) as u8;
            let rn_offset = get_triplet(value, 6);
            let rs = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            println!("Hello");
            println!("i: {}, op: {}", i, op);
            match (i, op) {
                (0, 0) => ThumbOpCode::ADD(InnerAdd::AddReg { rd, rs, rn: rn_offset }),
                (0, 1) => ThumbOpCode::ADD(InnerAdd::AddImm { rd, rs, offset: rn_offset }),
                (1, 0) => ThumbOpCode::SUB(InnerSub::SubReg { rd, rs, rn: rn_offset }),
                (1, 1) => ThumbOpCode::SUB(InnerSub::SubImm { rd, rs, offset: rn_offset }),
                // we're only testing 2 bits so anything other than above is impossible
                (_, _) => unreachable!(),
            }
        } else if value & 0xe000 == 0x0 {
            let opcode: u8 = (value >> 11 & 0x3) as u8;
            let offset: u8 = (value >> 6 & 0x1f) as u8;
            let rs: u8 = (value >> 3 & 0x7) as u8;
            let rd: u8 = (value & 0x7) as u8;
            match opcode {
                0 => ThumbOpCode::LSL(InnerShift::Reg { offset, rs, rd }),
                1 => ThumbOpCode::LSR(InnerShift::Reg { offset, rs, rd }),
                2 => ThumbOpCode::ASR(InnerShift::Reg { offset, rs, rd }),
                // The bits in opcode can be 0x3 but that means it isn't
                // an LSL, LSR, or ASR it is an ADD/SUB
                _ => unreachable!()
            }
        } else if value & 0xe000 == 0x2000 {
            let op = (value >> 11 & 0x3) as u8; 
            let rd = get_triplet(value, 8);
            let offset = (value & 0xff) as u8;
            match op {
                0 => ThumbOpCode::MOV(InnerMov::Offset{rd, offset}),
                1 => ThumbOpCode::CMP(InnerCmp::CmpByteImm{rd, offset}),
                2 => ThumbOpCode::ADD(InnerAdd::AddByteImm { rd, offset }),
                3 => ThumbOpCode::SUB(InnerSub::SubByteImm { rd, offset }),
                _ => unreachable!()
            }
        } else if value & 0xfc00 == 0x2000 {
            let op = (value >> 6 & 0xf) as u8;
            let rs = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match op {
                0 => ThumbOpCode::AND{rd, rs},
                1 => ThumbOpCode::EOR{rd, rs},
                2 => ThumbOpCode::LSL(InnerShift::Alu { rs, rd }),
                3 => ThumbOpCode::LSR(InnerShift::Alu { rs, rd }),
                4 => ThumbOpCode::ASR(InnerShift::Alu { rs, rd }),
                5 => ThumbOpCode::ADC{rd, rs},
                6 => ThumbOpCode::SBC{rd, rs},
                7 => ThumbOpCode::ROR{rd, rs},
                8 => ThumbOpCode::TST{rd, rs},
                9 => ThumbOpCode::NEG{rd, rs},
                10 => ThumbOpCode::CMP(InnerCmp::CmpAlu{rd, rs}),
                11 => ThumbOpCode::CMN{rd, rs},
                12 => ThumbOpCode::ORR{rd, rs},
                13 => ThumbOpCode::MUL{rd, rs},
                14 => ThumbOpCode::BIC{rd, rs},
                15 => ThumbOpCode::MVN{rd, rs},
                _ => unreachable!()
            }
        } else if value & 0xfc00 == 0x8800 {
            let op = (value >> 8 & 0x3) as u8;
            let h1 = (value >> 7 & 1) as u8;
            let h2 = (value >> 6 & 1) as u8;
            let rs = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match (op, h1, h2) {
                (_, _, _) => unreachable!()
            }

        } else {
            todo!()
        }
    }
}

mod test {
    use super::*;

    #[test]
    fn test_lsl_decode() {
        let inst: u16 = 0x0636;
        let op = ThumbOpCode::from(inst);
        assert_eq!(op, ThumbOpCode::LSL(InnerShift::Reg { rd: 6, rs: 6, offset: 0x18 }));
    }

    #[test]
    fn test_add_reg_variant() {
        let inst: u16 = 0x19ad;
        let op = ThumbOpCode::from(inst);
        assert_eq!(op, ThumbOpCode::ADD(InnerAdd::AddReg { rd: 5, rs: 5, rn: 6 }))
    }

    #[test]
    fn test_sub_imm_variant() {
        let inst: u16 = 0x1e68;
        let op = ThumbOpCode::from(inst);
        assert_eq!(op, ThumbOpCode::SUB(InnerSub::SubImm { rd: 0, rs: 5, offset: 1 }))
    }

    #[test]
    fn test_mov_imm_variant() {
        let inst: u16 = 0x2400;
        let op = ThumbOpCode::from(inst);
        assert_eq!(op, ThumbOpCode::MOV(InnerMov::Offset{rd: 4, offset: 0}));
    }

    #[test]
    fn test_add_byte_imm_vairant() {
        let inst = 0x3210;
        let op = ThumbOpCode::from(inst);
        assert_eq!(op, ThumbOpCode::ADD(InnerAdd::AddByteImm { rd: 2, offset: 0x10 }))
    }
}
