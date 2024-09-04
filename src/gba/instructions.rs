#[derive(Debug, strum_macros::Display, PartialEq)]
pub enum Conditional {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    #[strum(to_string = "")]
    AL,
    NV,
}

impl From<u32> for Conditional {
    fn from(instruction: u32) -> Self {
        let conditional = instruction >> 28;
        match conditional {
            0 => Conditional::EQ,
            1 => Conditional::NE,
            2 => Conditional::CS,
            3 => Conditional::CC,
            4 => Conditional::MI,
            5 => Conditional::PL,
            6 => Conditional::VS,
            7 => Conditional::VC,
            8 => Conditional::HI,
            9 => Conditional::LS,
            10 => Conditional::GE,
            11 => Conditional::LT,
            12 => Conditional::GT,
            13 => Conditional::LE,
            14 => Conditional::AL,
            _ => Conditional::NV,
        }
    }
}

// S might be better place in the Enum, rather than the op struct
#[derive(Debug, strum_macros::Display, PartialEq)]
pub enum Opcode {
    AND(Conditional, DataProcessingOp),
    EOR(Conditional, DataProcessingOp),
    SUB(Conditional, DataProcessingOp),
    RSB(Conditional, DataProcessingOp),
    ADD(Conditional, DataProcessingOp),
    ADC(Conditional, DataProcessingOp),
    SBC(Conditional, DataProcessingOp),
    RSC(Conditional, DataProcessingOp),
    TST(Conditional, DataProcessingOp),
    TEQ(Conditional, DataProcessingOp),
    CMP(Conditional, DataProcessingOp),
    CMN(Conditional, DataProcessingOp),
    ORR(Conditional, DataProcessingOp),
    MOV(Conditional, DataProcessingOp),
    BIC(Conditional, DataProcessingOp),
    MVN(Conditional, DataProcessingOp),
    MUL(Conditional, MultiplyOp),
    MLA(Conditional, MultiplyOp),
    // TODO: Change these to UMULL, UMLAL, SMULL, SMLAL
    MULL(Conditional, MultiplyLongOp),
    MLAL(Conditional, MultiplyLongOp),
    SWP(Conditional, SingleDataSwapOp),
    B(Conditional, BranchOp),
    BL(Conditional, BranchOp),
    BX(Conditional, BranchExchangeOp),
    SWI(Conditional),
    LDR(Conditional, SingleDataTfx),
    STR(Conditional, SingleDataTfx),
    LDM(Conditional, BlockDataTransfer),
    STM(Conditional, BlockDataTransfer),
    CDP(Conditional, CoprocessDataOp),
    LDC(Conditional, CoprocessDataTfx),
    STC(Conditional, CoprocessDataTfx),
    MRC(Conditional, CoprocessRegTfx),
    MCR(Conditional, CoprocessRegTfx),
    // TODO: Implement Half-word opcodes
    #[strum(to_string = "Undefined: {0}")]
    Undef(u32),
}

impl From<u32> for Opcode {
    fn from(inst: u32) -> Self {
        let cond = Conditional::from(inst);
        if is_data_processing(inst) {
            let op = DataProcessingOp::from(inst);
            let code = (inst >> 21) & 0xf;
            match code {
                0 => Opcode::AND(cond, op),
                1 => Opcode::EOR(cond, op),
                2 => Opcode::SUB(cond, op),
                3 => Opcode::RSB(cond, op),
                4 => Opcode::ADD(cond, op),
                5 => Opcode::ADC(cond, op),
                6 => Opcode::SBC(cond, op),
                7 => Opcode::RSC(cond, op),
                8 => Opcode::TST(cond, op),
                9 => Opcode::TEQ(cond, op),
                10 => Opcode::CMP(cond, op),
                11 => Opcode::CMN(cond, op),
                12 => Opcode::ORR(cond, op),
                13 => Opcode::MOV(cond, op),
                14 => Opcode::BIC(cond, op),
                _ => Opcode::MVN(cond, op),
            }
        } else if is_multiply(inst) {
            let op = MultiplyOp::from(inst);
            if op.a {
                Opcode::MLA(cond, op)
            } else {
                Opcode::MUL(cond, op)
            }
        } else if is_multiply_long(inst) {
            let op = MultiplyLongOp::from(inst);
            if op.a {
                Opcode::MLAL(cond, op)
            } else {
                Opcode::MULL(cond, op)
            }
        } else if is_single_data_swap(inst) {
            let op = SingleDataSwapOp::from(inst);
            Opcode::SWP(cond, op)
        } else if is_branch_and_exchange(inst) {
            let op = BranchExchangeOp::from(inst);
            Opcode::BX(cond, op)
        } else if is_branch(inst) {
            let op = BranchOp::from(inst);
            if op.l {
                Opcode::BL(cond, op)
            } else {
                Opcode::B(cond, op)
            }
        } else if is_software_interrupt(inst) {
            Opcode::SWI(cond)
        } else if is_single_data_tfx(inst) {
            let op = SingleDataTfx::from(inst);
            if op.l {
                Opcode::LDR(cond, op)
            } else {
                Opcode::STR(cond, op)
            }
        } else if is_block_data_tfx(inst) {
            let op = BlockDataTransfer::from(inst);
            if op.l {
                Opcode::LDM(cond, op)
            } else {
                Opcode::STM(cond, op)
            }
        } else if is_coprocessor_data_op(inst) {
            let op = CoprocessDataOp::from(inst);
            Opcode::CDP(cond, op)
        } else if is_coprocessor_data_tfx(inst) {
            let op = CoprocessDataTfx::from(inst);
            if op.l {
                Opcode::LDC(cond, op)
            } else {
                Opcode::STC(cond, op)
            }
        } else if is_coprocessor_reg_tfx(inst) {
            let op = CoprocessRegTfx::from(inst);
            if op.l {
                Opcode::MRC(cond, op)
            } else {
                Opcode::MCR(cond, op)
            }
        } else {
            Opcode::Undef(inst)
        }
    }
}

impl Opcode {
    pub fn string_repr(&self) -> String {
        match self {
            Opcode::AND(c, o)
            | Opcode::EOR(c, o)
            | Opcode::ORR(c, o)
            | Opcode::BIC(c, o)
            | Opcode::ADD(c, o)
            | Opcode::SUB(c, o)
            | Opcode::ADC(c, o)
            | Opcode::SBC(c, o)
            | Opcode::RSC(c, o)
            | Opcode::RSB(c, o) => {
                format!("{}{} {} r{} r{}, <{:#x}>", self, c, o.s, o.rd, o.rn, o.operand)
            }
            Opcode::TST(c, o) | Opcode::TEQ(c, o) => {
                format!("{}{} r{}, <{:#x}>", self, c, o.rn, o.operand)
            }
            Opcode::CMP(c, o) | Opcode::CMN(c, o) => {
                format!("{}{} r{}, <{:#x}>", self, c, o.rd, o.operand)
            }
            Opcode::MOV(c, o) | Opcode::MVN(c, o) => {
                format!("{}{} {} r{}, <{:#x}>", self, c, o.s, o.rd, o.operand)
            }
            Opcode::MLA(c, o) => {
                format!(
                    "{}{} {} r{}, r{}, r{}, r{}",
                    self, c, o.s, o.rd, o.rm, o.rs, o.rn
                )
            }
            Opcode::MUL(c, o) => {
                format!("{}{} {} r{}, r{}, r{}", self, c, o.s, o.rd, o.rm, o.rs)
            }
            Opcode::MULL(c, o) | Opcode::MLAL(c, o) => {
                format!(
                    "{}{} {} r{}, r{}, r{}, r{}",
                    self, c, o.s, o.rd_hi, o.rd_lo, o.rm, o.rs
                )
            }
            Opcode::B(c, o) | Opcode::BL(c, o) => {
                format!("{}{} +{:#x}", self, c, o.offset)
            }
            Opcode::BX(c, o) => {
                format!("{}{} r{}", self, c, o.rn)
            }
            Opcode::SWP(c, o) => {
                format!("{}{} {} r{}, r{}, r{}", self, c, o.b, o.rd, o.rm, o.rn)
            }
            // TODO: Expand this
            Opcode::LDM(c, _) | Opcode::STM(c, _) => {
                // TODO: This is actually more complicated
                format!("{}{}", self, c)
            }
            // TODO: Expand this
            Opcode::LDR(c, _) | Opcode::STR(c, _) |
            Opcode::CDP(c, _) | Opcode::LDC(c, _) |
            Opcode::STC(c, _) | Opcode::MRC(c, _) |
            Opcode::MCR(c, _) => {
                // TODO: This is actually more complicated
                format!("{}{}", self, c)
            }
            Opcode::SWI(c) => {
                format!("{}{}", self, c)
            },
            Opcode::Undef(_) => {
                format!("undefined")
            }
        }
    }
}

// TODO: Maybe rename this to DataOperation and use other structs
// like branch operation
#[derive(Debug, PartialEq)]
pub struct DataProcessingOp {
    s: bool,
    rn: u8,
    rd: u8,
    operand: u16,
}

impl From<u32> for DataProcessingOp {
    fn from(inst: u32) -> Self {
        DataProcessingOp {
            s: (inst >> 20 & 0x1) == 0x1,
            rd: (inst >> 12 & 0xf) as u8,
            rn: (inst >> 16 & 0xf) as u8,
            operand: (inst & 0xfff) as u16,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MultiplyOp {
    // whether operation should accumulate
    a: bool,
    s: bool,
    rd: u8,
    rn: u8,
    rs: u8,
    rm: u8,
}

impl From<u32> for MultiplyOp {
    fn from(inst: u32) -> Self {
        Self {
            a: (inst >> 21 & 0x1) == 0x1,
            s: (inst >> 20 & 0x1) == 0x1,
            rd: (inst >> 16 & 0xf) as u8,
            rn: (inst >> 12 & 0xf) as u8,
            rs: (inst >> 8 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MultiplyLongOp {
    // whether operation is signed or unsigned
    u: bool,
    // whether opeartion should accumulate
    a: bool,
    s: bool,
    rd_hi: u8,
    rd_lo: u8,
    rs: u8,
    rm: u8,
}

impl From<u32> for MultiplyLongOp {
    fn from(inst: u32) -> Self {
        Self {
            u: (inst >> 22 & 0x1) == 0x1,
            a: (inst >> 21 & 0x1) == 0x1,
            s: (inst >> 20 & 0x1) == 0x1,
            rd_hi: (inst >> 16 & 0xf) as u8,
            rd_lo: (inst >> 12 & 0xf) as u8,
            rs: (inst >> 8 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SingleDataSwapOp {
    b: bool,
    rn: u8,
    rd: u8,
    rm: u8,
}

impl From<u32> for SingleDataSwapOp {
    fn from(inst: u32) -> Self {
        Self {
            b: (inst >> 22 & 0x1) == 0x1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BranchExchangeOp {
    rn: u8,
}

impl From<u32> for BranchExchangeOp {
    fn from(inst: u32) -> Self {
        Self {
            rn: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BranchOp {
    l: bool,
    offset: u32,
}

impl From<u32> for BranchOp {
    fn from(inst: u32) -> Self {
        Self {
            l: (inst >> 25 & 0x1) == 0x1,
            offset: (inst & 0xffffff) as u32,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HalfwordRegOffset {
    p: bool,
    u: bool,
    w: bool,
    l: bool,
    s: bool,
    h: bool,
    rn: u8,
    rd: u8,
    rm: u8,
}

impl From<u32> for HalfwordRegOffset {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            s: (inst >> 6 & 1) == 1,
            h: (inst >> 5 & 1) == 1,
            rn: (inst >> 15 & 0xf) as u8,
            rd: (inst >> 11 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HalfwordImmOffset {
    p: bool,
    u: bool,
    w: bool,
    l: bool,
    s: bool,
    h: bool,
    rn: u8,
    rd: u8,
    offset_a: u8,
    offset_b: u8,
}

impl From<u32> for HalfwordImmOffset {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            s: (inst >> 6 & 1) == 1,
            h: (inst >> 5 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            offset_a: (inst >> 8 & 0xf) as u8,
            offset_b: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SingleDataTfx {
    i: bool,
    p: bool,
    u: bool,
    b: bool,
    w: bool,
    l: bool,
    rn: u8,
    rd: u8,
    offset: u16,
}

impl From<u32> for SingleDataTfx {
    fn from(inst: u32) -> Self {
        Self {
            i: (inst >> 25 & 1) == 1,
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            b: (inst >> 22 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            offset: (inst & 0xfff) as u16,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BlockDataTransfer {
    p: bool,
    u: bool,
    s: bool,
    w: bool,
    l: bool,
    rn: u8,
    register_list: u16,
}

impl From<u32> for BlockDataTransfer {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            s: (inst >> 22 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            register_list: (inst & 0xffff) as u16,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CoprocessDataTfx {
    p: bool,
    u: bool,
    n: bool,
    w: bool,
    l: bool,
    rn: u8,
    c_rd: u8,
    cp_num: u8,
    offset: u16,
}

impl From<u32> for CoprocessDataTfx {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            n: (inst >> 22 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            c_rd: (inst >> 12 & 0xf) as u8,
            cp_num: (inst >> 8 & 0xf) as u8,
            offset: (inst & 0xffff) as u16,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CoprocessDataOp {
    cp_opc: u8,
    c_rn: u8,
    c_rd: u8,
    cp_num: u8,
    cp: u8,
    c_rm: u8,
}

impl From<u32> for CoprocessDataOp {
    fn from(inst: u32) -> Self {
        Self {
            cp_opc: (inst >> 20 & 0xf) as u8,
            c_rn: (inst >> 16 & 0xf) as u8,
            c_rd: (inst >> 12 & 0xf) as u8,
            cp_num: (inst >> 8 & 0xf) as u8,
            cp: (inst >> 5 & 0x7) as u8,
            c_rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CoprocessRegTfx {
    l: bool,
    cp_opc: u8,
    c_rn: u8,
    c_rd: u8,
    cp_num: u8,
    cp: u8,
    c_rm: u8,
}

impl From<u32> for CoprocessRegTfx {
    fn from(inst: u32) -> Self {
        Self {
            l: (inst >> 20 & 1) == 1,
            cp_opc: (inst >> 21 & 0xf) as u8,
            c_rn: (inst >> 16 & 0xf) as u8,
            c_rd: (inst >> 12 & 0xf) as u8,
            cp_num: (inst >> 8 & 0xf) as u8,
            cp: (inst >> 5 & 0x7) as u8,
            c_rm: (inst & 0xf) as u8,
        }
    }
}

pub fn is_data_processing(inst: u32) -> bool {
    inst & 0x0e000000 == 0x02000000
}

pub fn is_multiply(inst: u32) -> bool {
    inst & 0x0fc000f0 == 0x00000090
}

pub fn is_multiply_long(inst: u32) -> bool {
    inst & 0x0fc000f0 == 0x00800090
}

pub fn is_single_data_swap(inst: u32) -> bool {
    inst & 0x0fb00ff0 == 0x01000090
}

pub fn is_branch_and_exchange(inst: u32) -> bool {
    inst & 0x0ffffff0 == 0x012fff10
}

pub fn is_halfword_data_tfx_reg(inst: u32) -> bool {
   inst & 0x0e400f90 == 0x00000090
}

pub fn is_halfword_data_tfx_imm(inst: u32) -> bool {
    inst & 0x0e400090 == 0x00400090
}

pub fn is_single_data_tfx(inst: u32) -> bool {
    inst & 0x0c000000 == 0x04000000
}

pub fn is_undefined(inst: u32) -> bool {
    inst & 0x0e000010 == 0x06000010
}

pub fn is_block_data_tfx(inst: u32) -> bool {
    inst & 0x0e000000 == 0x08000000
}

pub fn is_branch(inst: u32) -> bool {
    inst & 0x0e000000 == 0x0a000000
}

pub fn is_coprocessor_data_tfx(inst: u32) -> bool {
    inst & 0x0e000000 == 0x0c000000
}

pub fn is_coprocessor_data_op(inst: u32) -> bool {
    inst & 0x0f000010 == 0x0c000000
}

pub fn is_coprocessor_reg_tfx(inst: u32) -> bool {
    inst & 0x0f000010 == 0x0c000010
}

pub fn is_software_interrupt(inst: u32) -> bool {
    inst & 0x0f000000 == 0x0f000000
}

mod test {
    use super::*;

    #[test]
    fn test_branch_check() {
        let inst: u32 = 0b11101010000000000000000000011000;
        let branch = is_branch(inst);
        assert_eq!(branch, true);
    }

    #[test]
    fn test_branch_decode() {
        let inst: u32 = 0b11101010000000000000000000011000;
        let op = Opcode::from(inst);
        let op2 = Opcode::B(
            Conditional::AL,
            BranchOp {
                l: false,
                offset: 0b11000,
            },
        );
        assert_eq!(op, op2);
    }
}
