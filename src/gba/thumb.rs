use super::Operation;
use crate::{SystemMemory, CPU};

#[derive(Debug, PartialEq)]
pub enum ThumbInstruction {
    ADC{rd: u8, rs: u8},
    ADD(InnerAdd),
    AND{rd: u8, rs: u8},
    ASR(InnerShift),
    B(u16),
    // TODO: In Ghidra BL is almost always 4 bytes instead of 2
    BL{h: bool, offset: u8},
    BEQ(u8),
    BNE(u8),
    BCS(u8),
    BCC(u8),
    BMI(u8),
    BPL(u8),
    BVS(u8),
    BVC(u8),
    BHI(u8),
    BLS(u8),
    BGE(u8),
    BLT(u8),
    BGT(u8),
    BLE(u8),
    BIC{rd: u8, rs: u8},
    BX(InnerBranchEx),
    CMN{rd: u8, rs: u8},
    CMP(InnerCmp),
    EOR{rd: u8, rs: u8},
    LDMIA{rb: u8, r_list: u8},
    LDR(InnerLdr),
    LDRB(InnerStoreLoadByte),
    LDRH(InnerStoreLoadByte),
    LSL(InnerShift),
    LDSB{rd: u8, rb: u8, ro: u8},
    LDSH{rd: u8, rb: u8, ro: u8},
    LSR(InnerShift),
    MOV(InnerMov),
    MUL{rd: u8, rs: u8},
    MVN{rd: u8, rs: u8},
    NEG{rd: u8, rs: u8},
    ORR{rd: u8, rs: u8},
    POP(InnerStack),
    PUSH(InnerStack),
    ROR{rd: u8, rs: u8},
    SBC{rd: u8, rs: u8},
    STMIA{rb: u8, r_list: u8},
    STR(InnerStr),
    STRB(InnerStoreLoadByte),
    STRH(InnerStoreLoadByte),
    SWI(u8),
    SUB(InnerSub),
    TST{rd: u8, rs: u8},
    Undefined,
} 

#[derive(Debug, PartialEq)]
pub enum InnerAdd {
    AddReg{rd: u8, rs: u8, rn: u8},
    AddImm{rd: u8, rs: u8, offset: u8},
    AddByteImm{rd: u8, offset: u8},
    AddLowToHi{hd: u8, rs: u8},
    AddHiToLow{rd: u8, hs: u8},
    AddHiToHi{hd: u8, hs: u8},
    PC {rd: u8, word: u16},
    Sp10Bit {rd: u8, word: u16},
    Sp9Bit {word: u16},
    SpNeg {word: u16}
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

#[derive(Debug, PartialEq)]
pub enum InnerLdr {
    PC {rd: u8, word: u8},
    Reg {rd: u8, rb: u8, ro: u8},
    Offset {offset: u8, rb: u8, rd: u8},
    SP {rd: u8, word: u16},
}

#[derive(Debug, PartialEq)]
pub enum InnerStr {
    Reg {rd: u8, rb: u8, ro: u8},
    Offset {offset: u8, rb: u8, rd: u8},
    SP {rd: u8, word: u16},
}

#[derive(Debug, PartialEq)]
pub enum InnerStoreLoadByte {
    Reg {rd: u8, rb: u8, ro: u8},
    Offset {offset: u8, rb: u8, rd: u8}
}

#[derive(Debug, PartialEq)]
pub enum InnerStack {
    // This is a bit map push/pop r4 if bit 4 is set to 1
    RList(u8),
    // The same as above + LR for push or PC for pop
    LrPc(u8)
}

fn get_triplet(value: u16, shift: u32) -> u8 {
    (value >> shift & 0x7) as u8
}

impl From<u32> for ThumbInstruction {
    // Takes the bottom half-word to create an opcode
    // callers should shift their u32 to deocde the correct half
    fn from(value: u32) -> Self {
        let lower_half = (value & 0xffff) as u16;
        ThumbInstruction::from(lower_half)
    }
}

impl From<u16> for ThumbInstruction {
    fn from(value: u16) -> Self {
        if value & 0xf800 == 0x1800 {
            let i: u8 = (value >> 10 & 0x1) as u8;
            let op: u8 = (value >> 9 & 0x1) as u8;
            let rn_offset = get_triplet(value, 6);
            let rs = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match (i, op) {
                (0, 0) => ThumbInstruction::ADD(InnerAdd::AddReg { rd, rs, rn: rn_offset }),
                (0, 1) => ThumbInstruction::SUB(InnerSub::SubReg { rd, rs, rn: rn_offset }),
                (1, 0) => ThumbInstruction::ADD(InnerAdd::AddImm { rd, rs, offset: rn_offset }),
                (1, 1) => ThumbInstruction::SUB(InnerSub::SubImm { rd, rs, offset: rn_offset }),
                // we're only testing 2 bits so anything other than above is impossible
                (_, _) => unreachable!(),
            }
        } else if value & 0xe000 == 0x0 {
            let opcode: u8 = (value >> 11 & 0x3) as u8;
            let offset: u8 = (value >> 6 & 0x1f) as u8;
            let rs: u8 = (value >> 3 & 0x7) as u8;
            let rd: u8 = (value & 0x7) as u8;
            match opcode {
                0 => ThumbInstruction::LSL(InnerShift::Reg { offset, rs, rd }),
                1 => ThumbInstruction::LSR(InnerShift::Reg { offset, rs, rd }),
                2 => ThumbInstruction::ASR(InnerShift::Reg { offset, rs, rd }),
                // The bits in opcode can be 0x3 but that means it isn't
                // an LSL, LSR, or ASR it is an ADD/SUB
                _ => unreachable!()
            }
        } else if value & 0xe000 == 0x2000 {
            let op = (value >> 11 & 0x3) as u8; 
            let rd = get_triplet(value, 8);
            let offset = (value & 0xff) as u8;
            match op {
                0 => ThumbInstruction::MOV(InnerMov::Offset{rd, offset}),
                1 => ThumbInstruction::CMP(InnerCmp::CmpByteImm{rd, offset}),
                2 => ThumbInstruction::ADD(InnerAdd::AddByteImm { rd, offset }),
                3 => ThumbInstruction::SUB(InnerSub::SubByteImm { rd, offset }),
                _ => unreachable!()
            }
        } else if value & 0xfc00 == 0x2000 {
            let op = (value >> 6 & 0xf) as u8;
            let rs = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match op {
                0 => ThumbInstruction::AND{rd, rs},
                1 => ThumbInstruction::EOR{rd, rs},
                2 => ThumbInstruction::LSL(InnerShift::Alu { rs, rd }),
                3 => ThumbInstruction::LSR(InnerShift::Alu { rs, rd }),
                4 => ThumbInstruction::ASR(InnerShift::Alu { rs, rd }),
                5 => ThumbInstruction::ADC{rd, rs},
                6 => ThumbInstruction::SBC{rd, rs},
                7 => ThumbInstruction::ROR{rd, rs},
                8 => ThumbInstruction::TST{rd, rs},
                9 => ThumbInstruction::NEG{rd, rs},
                10 => ThumbInstruction::CMP(InnerCmp::CmpAlu{rd, rs}),
                11 => ThumbInstruction::CMN{rd, rs},
                12 => ThumbInstruction::ORR{rd, rs},
                13 => ThumbInstruction::MUL{rd, rs},
                14 => ThumbInstruction::BIC{rd, rs},
                15 => ThumbInstruction::MVN{rd, rs},
                _ => unreachable!()
            }
        } else if value & 0xfc00 == 0x4400 {
            let op = (value >> 8 & 0x3) as u8;
            let h1 = (value >> 7 & 1) as u8;
            let h2 = (value >> 6 & 1) as u8;
            let rs = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match (op, h1, h2) {
                (0, 0, 1) => ThumbInstruction::ADD(InnerAdd::AddHiToLow { rd, hs: rs + 8 }),
                (0, 1, 0) => ThumbInstruction::ADD(InnerAdd::AddLowToHi { hd: rd + 8, rs }),
                (0, 1, 1) => ThumbInstruction::ADD(InnerAdd::AddHiToHi { hd: rd + 8, hs: rs + 8}),
                (1, 0, 1) => ThumbInstruction::CMP(InnerCmp::CmpHiToLow { rd, hs: rs + 8}),
                (1, 1, 0) => ThumbInstruction::CMP(InnerCmp::CmpLowToHi  { hd: rd + 8, rs }),
                (1, 1, 1) => ThumbInstruction::CMP(InnerCmp::CmpHiToHi { hd: rd + 8, hs: rs + 8}),
                (2, 0, 1) => ThumbInstruction::MOV(InnerMov::HiToLow { rd, hs: rs + 8}),
                (2, 1, 0) => ThumbInstruction::MOV(InnerMov::LowToHi { hd: rd + 8, rs }),
                (2, 1, 1) => ThumbInstruction::MOV(InnerMov::HiToHi { hd: rd + 8, hs: rs + 8}),
                (3, 0, 0) => ThumbInstruction::BX(InnerBranchEx::Low { rs }),
                (3, 0, 1) => ThumbInstruction::BX(InnerBranchEx::Hi { hs: rs + 8 }),
                (_, _, _) => unreachable!()
            }
        } else if value & 0xf800 == 0x4800 {
            let rd = get_triplet(value, 8);
            let offset = (value & 0xff) as u8;
            ThumbInstruction::LDR(InnerLdr::PC { rd, word: offset })
        } else if value & 0xf200 == 0x5000 {
            let l = (value >> 11 & 1) == 1;
            let b = (value >> 10 & 1) == 1;
            let ro = get_triplet(value, 6);
            let rb = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match (l, b) {
                (false, false) => ThumbInstruction::STR(InnerStr::Reg{rd, rb, ro}),
                (false, true) => ThumbInstruction::STRB(InnerStoreLoadByte::Reg{ rd, rb ,ro }),
                (true, false) => ThumbInstruction::LDR(InnerLdr::Reg { rd, rb, ro }),
                (true, true) => ThumbInstruction::LDRB(InnerStoreLoadByte::Reg { rd, rb, ro }),
            }

        } else if value & 0xf200 == 0x5200 {
            let h = (value >> 11 & 1) == 1;
            let s = (value >> 10 & 1) == 1;
            let ro = get_triplet(value, 6);
            let rb = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match (h, s) {
                (false, false) => ThumbInstruction::STRH(InnerStoreLoadByte::Reg{ rd, rb, ro }),
                (false, true) => ThumbInstruction::LDRH(InnerStoreLoadByte::Reg{ rd, rb ,ro }),
                (true, false) => ThumbInstruction::LDSB { rd, rb, ro },
                (true, true) => ThumbInstruction::LDSH { rd, rb, ro },
            }
        } else if value & 0xe000 == 0x6000 {
            let b = (value >> 12 & 1) == 1;
            let l = (value >> 11 & 1) == 1;
            let offset = (value >> 6 & 0x1f) as u8;
            let rb = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            match (l, b) {
                (false, false) => ThumbInstruction::STR(InnerStr::Offset { offset, rb, rd }),
                (false, true) => ThumbInstruction::STRB(InnerStoreLoadByte::Offset { offset, rb, rd }),
                (true, false) => ThumbInstruction::LDR(InnerLdr::Offset { offset, rb, rd }),
                (true, true) => ThumbInstruction::LDRB(InnerStoreLoadByte::Offset { offset, rb, rd }),
            }
        } else if value & 0xf000 == 0x8000 {
            let l = (value >> 11 & 1) == 1;
            let offset = (value >> 5 & 0x3e) as u8;
            let rb = get_triplet(value, 3);
            let rd = get_triplet(value, 0);
            if l {
                ThumbInstruction::LDRH(InnerStoreLoadByte::Offset { offset, rb, rd })
            } else {
                ThumbInstruction::STRH(InnerStoreLoadByte::Offset { offset, rb, rd })
            }
        } else if value & 0xf000 == 0x9000 {
            let l = (value >> 11 & 1) == 1;
            let rd = get_triplet(value, 8);
            let word = (value & 0xff << 2) as u16;
            if l {
                ThumbInstruction::LDR(InnerLdr::SP { word, rd })
            } else {
                ThumbInstruction::STR(InnerStr::SP { word, rd })
            }
        } else if value & 0xf000 == 0xa000 {
            let sp = (value >> 11 & 0x1) == 1;
            let rd = get_triplet(value, 8);
            let word = (value & 0xff << 2) as u16;
            if sp {
                ThumbInstruction::ADD(InnerAdd::Sp10Bit { rd, word })
            } else {
                ThumbInstruction::ADD(InnerAdd::PC { rd, word })
            }
        } else if value & 0xff00 == 0xb000 {
            let s = (value >> 7 & 1) == 1;
            let word = (value & 0x7f << 2) as u16;
            if s {
                ThumbInstruction::ADD(InnerAdd::Sp9Bit { word })
            } else {
                ThumbInstruction::ADD(InnerAdd::SpNeg { word })
            }
        } else if value & 0xf600 == 0xb400 {
            let l = value >> 11 & 1;
            let r = value >> 8 & 1;
            let r_list = (value & 0xff) as u8;
            match (l, r) {
                (0, 0) => ThumbInstruction::PUSH(InnerStack::RList(r_list)),
                (0, 1) => ThumbInstruction::PUSH(InnerStack::LrPc(r_list)),
                (1, 0) => ThumbInstruction::POP(InnerStack::RList(r_list)),
                (1, 1) => ThumbInstruction::POP(InnerStack::LrPc(r_list)),
                (_, _) => unreachable!()
            }
        } else if value & 0xf000 == 0xc000 {
            let l = (value >> 11 & 1) == 1;
            let rb = get_triplet(value, 8);
            let r_list = (value & 0xff) as u8;
            if l {
                ThumbInstruction::LDMIA { rb, r_list }
            } else {
                ThumbInstruction::STMIA { rb, r_list }
            }
        } else if value & 0xf000 == 0xd000 {
            let cond = (value >> 8 & 0xf) as u8;
            let offset = (value & 0xff) as u8;
            match cond {
                0 => ThumbInstruction::BEQ(offset),
                1 => ThumbInstruction::BNE(offset),
                2 => ThumbInstruction::BCS(offset),
                3 => ThumbInstruction::BCC(offset),
                4 => ThumbInstruction::BMI(offset),
                5 => ThumbInstruction::BPL(offset),
                6 => ThumbInstruction::BVS(offset),
                7 => ThumbInstruction::BVC(offset),
                8 => ThumbInstruction::BHI(offset),
                9 => ThumbInstruction::BLS(offset),
                10 => ThumbInstruction::BGE(offset),
                11 => ThumbInstruction::BLT(offset),
                12 => ThumbInstruction::BGT(offset),
                13 => ThumbInstruction::BLE(offset),
                15 => ThumbInstruction::SWI(offset),
                _ => unreachable!()
            }
        } else if value & 0xf800 == 0xe000 {
            let offset = (value & 0x7ff) << 1;
            ThumbInstruction::B(offset)
        } else if value & 0xf000 == 0xf000 {
            let h = (value >> 11 & 1) == 1;
            let offset = (value & 0x7ff) as u8;
            ThumbInstruction::BL{h, offset}
        } else {
            ThumbInstruction::Undefined
        }
    }
}

#[derive(Debug)]
pub struct Undefined;

impl Operation for Undefined {
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
        unreachable!()
    }
}

pub fn decode_as_thumb(_inst: u32) -> Box<dyn Operation> {
    Box::new(Undefined)
}

mod test {
    use super::*;

    #[test]
    fn test_lsl_decode() {
        let inst: u16 = 0x0636;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::LSL(InnerShift::Reg { rd: 6, rs: 6, offset: 0x18 }));
    }

    #[test]
    fn test_add_reg_variant() {
        let inst: u16 = 0x19ad;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::ADD(InnerAdd::AddReg { rd: 5, rs: 5, rn: 6 }))
    }

    #[test]
    fn test_sub_imm_variant() {
        let inst: u16 = 0x1e68;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::SUB(InnerSub::SubImm { rd: 0, rs: 5, offset: 1 }))
    }

    #[test]
    fn test_add_imm_variant() {
        let inst: u16 = 0x1c22;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::ADD(InnerAdd::AddImm { rd: 2, rs: 4, offset: 0 }))
    }

    #[test]
    fn test_mov_imm_variant() {
        let inst: u16 = 0x2400;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::MOV(InnerMov::Offset{rd: 4, offset: 0}));
    }

    #[test]
    fn test_add_byte_imm_vairant() {
        let inst: u16 = 0x3210;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::ADD(InnerAdd::AddByteImm { rd: 2, offset: 0x10 }))
    }

    #[test]
    fn test_bx_variant_one() {
        let inst: u16 = 0x4770;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::BX(InnerBranchEx::Hi { hs: 14 }))
    }

    #[test]
    fn test_bx_variant_two() {
        let inst: u16 = 0x4718;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::BX(InnerBranchEx::Low { rs: 3 }))
    }

    #[test]
    fn test_ldr_decode() {
        let inst: u16 = 0x49f8;
        let op = ThumbInstruction::from(inst);
        // TODO: in ghidra this is DAT_0000ac0
        assert_eq!(op, ThumbInstruction::LDR(InnerLdr::PC{rd: 1, word: 0xf8}));
    }

    #[test]
    fn test_ldrb_decode() {
        let inst: u16 = 0x5d82;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::LDRB(InnerStoreLoadByte::Reg{ rd: 2, rb: 0, ro: 6 }));
    }

    #[test]
    fn test_strh_decode() {
        let inst: u16 = 0x81bb;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::STRH(InnerStoreLoadByte::Offset { offset: 0xc, rb: 7, rd: 3 }));
    }

    #[test]
    fn test_b_decode() {
        let inst: u16 = 0xe3a0;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::B(0x740));
    }

    #[test]
    fn test_push_decode() {
        let inst: u16 = 0xb578;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::PUSH(InnerStack::LrPc(0b1111000)));
    }

    #[test]
    fn test_strh_decode_two() {
        let inst: u16 = 0x7090;
        let op = ThumbInstruction::from(inst);
        assert_eq!(op, ThumbInstruction::STRB(InnerStoreLoadByte::Offset{rd: 0, rb: 2, offset: 2}));
    }
}
