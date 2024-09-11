// TODO: Possible alternative to this is to have all 15
// operation structs be a trait "Operable", that takes a CPU
// and modifies it based on it's instruction
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
pub enum ArmInstruction {
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
    UMULL(Conditional, MultiplyLongOp),
    SMULL(Conditional, MultiplyLongOp),
    UMLAL(Conditional, MultiplyLongOp),
    SMLAL(Conditional, MultiplyLongOp),
    SWP(Conditional, SingleDataSwapOp),
    SWPB(Conditional, SingleDataSwapOp),
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
    MRS(Conditional, PsrTransferOp),
    MSR(Conditional, PsrTransferOp),
    // TODO: Implement Half-word opcodes
    STRH(Conditional, HalfwordDataOp),
    LDRH(Conditional, HalfwordDataOp),
    LDRSB(Conditional, HalfwordDataOp),
    #[strum(to_string = "Undefined: {0}")]
    Undef(u32),
}

#[derive(Debug, PartialEq)]
pub enum AddressingMode3 {
    Imm { byte_offset: u8 },
    PreIndexedImm { byte_offset: u8 },
    PostIndexedImm { byte_offset: u8 },
    Reg { rm: u8 },
    PreIndexedReg { rm: u8 },
    PostIndexedReg { rm: u8 },

}

impl From<u32> for ArmInstruction {
    fn from(inst: u32) -> Self {
        let cond = Conditional::from(inst);
        if is_data_processing(inst) {
            let op = DataProcessingOp::from(inst);
            let code = (inst >> 21) & 0xf;
            match code {
                0 => ArmInstruction::AND(cond, op),
                1 => ArmInstruction::EOR(cond, op),
                2 => ArmInstruction::SUB(cond, op),
                3 => ArmInstruction::RSB(cond, op),
                4 => ArmInstruction::ADD(cond, op),
                5 => ArmInstruction::ADC(cond, op),
                6 => ArmInstruction::SBC(cond, op),
                7 => ArmInstruction::RSC(cond, op),
                8 => ArmInstruction::TST(cond, op),
                9 => ArmInstruction::TEQ(cond, op),
                10 => ArmInstruction::CMP(cond, op),
                11 => ArmInstruction::CMN(cond, op),
                12 => ArmInstruction::ORR(cond, op),
                13 => ArmInstruction::MOV(cond, op),
                14 => ArmInstruction::BIC(cond, op),
                _ => ArmInstruction::MVN(cond, op),
            }
        } else if is_multiply(inst) {
            let op = MultiplyOp::from(inst);
            if op.a {
                ArmInstruction::MLA(cond, op)
            } else {
                ArmInstruction::MUL(cond, op)
            }
        } else if is_multiply_long(inst) {
            let op = MultiplyLongOp::from(inst);
            if op.a {
                if op.s {
                    ArmInstruction::SMLAL(cond, op)
                } else {
                    ArmInstruction::UMLAL(cond, op)
                }
            } else {
                if op.s {
                    ArmInstruction::SMULL(cond, op)
                } else {
                    ArmInstruction::UMULL(cond, op)
                }
            }
        } else if is_single_data_swap(inst) {
            let op = SingleDataSwapOp::from(inst);
            if op.b {
                ArmInstruction::SWPB(cond, op)
            } else {
                ArmInstruction::SWP(cond, op)
            }
        } else if is_branch_and_exchange(inst) {
            let op = BranchExchangeOp::from(inst);
            ArmInstruction::BX(cond, op)
        } else if is_branch(inst) {
            let op = BranchOp::from(inst);
            if op.l {
                ArmInstruction::BL(cond, op)
            } else {
                ArmInstruction::B(cond, op)
            }
        } else if is_software_interrupt(inst) {
            ArmInstruction::SWI(cond)
        } else if is_single_data_tfx(inst) {
            let op = SingleDataTfx::from(inst);
            if op.l {
                ArmInstruction::LDR(cond, op)
            } else {
                ArmInstruction::STR(cond, op)
            }
        } else if is_block_data_tfx(inst) {
            let op = BlockDataTransfer::from(inst);
            if op.l {
                ArmInstruction::LDM(cond, op)
            } else {
                ArmInstruction::STM(cond, op)
            }
        } else if is_coprocessor_data_op(inst) {
            let op = CoprocessDataOp::from(inst);
            ArmInstruction::CDP(cond, op)
        } else if is_coprocessor_data_tfx(inst) {
            let op = CoprocessDataTfx::from(inst);
            if op.l {
                ArmInstruction::LDC(cond, op)
            } else {
                ArmInstruction::STC(cond, op)
            }
        } else if is_coprocessor_reg_tfx(inst) {
            let op = CoprocessRegTfx::from(inst);
            if op.l {
                ArmInstruction::MRC(cond, op)
            } else {
                ArmInstruction::MCR(cond, op)
            }
        } else if is_psr_transfer(inst) {
            let op = PsrTransferOp::from(inst);
            if is_mrs_op(inst) {
                ArmInstruction::MRS(cond, op)
            } else {
                ArmInstruction::MSR(cond, op)
            }
        } else if is_halfword_data_tfx_imm(inst) || is_halfword_data_tfx_reg(inst) {
            let op = HalfwordDataOp::from(inst);
            match (op.l, op.h) {
                (false, false) => unreachable!(),
                (false, true) => ArmInstruction::STRH(cond, op),
                (true, false) => ArmInstruction::LDRH(cond, op),
                (true, true) => ArmInstruction::LDRSB(cond, op),
            }
        } else {
            ArmInstruction::Undef(inst)
        }
    }
}

impl ArmInstruction {

    pub fn string_repr(&self) -> String {
        match self {
            ArmInstruction::AND(c, o)
            | ArmInstruction::EOR(c, o)
            | ArmInstruction::ORR(c, o)
            | ArmInstruction::BIC(c, o)
            | ArmInstruction::ADD(c, o)
            | ArmInstruction::SUB(c, o)
            | ArmInstruction::ADC(c, o)
            | ArmInstruction::SBC(c, o)
            | ArmInstruction::RSC(c, o)
            | ArmInstruction::RSB(c, o) => {
                format!("{}{} {} r{} r{}, <{:#x}>", self, c, o.s, o.rd, o.rn, o.operand)
            }
            ArmInstruction::TST(c, o) | ArmInstruction::TEQ(c, o) => {
                format!("{}{} r{}, <{:#x}>", self, c, o.rn, o.operand)
            }
            ArmInstruction::CMP(c, o) | ArmInstruction::CMN(c, o) => {
                format!("{}{} r{}, <{:#x}>", self, c, o.rd, o.operand)
            }
            ArmInstruction::MOV(c, o) | ArmInstruction::MVN(c, o) => {
                format!("{}{} {} r{}, <{:#x}>", self, c, o.s, o.rd, o.operand)
            }
            ArmInstruction::MLA(c, o) => {
                format!(
                    "{}{} {} r{}, r{}, r{}, r{}",
                    self, c, o.s, o.rd, o.rm, o.rs, o.rn
                )
            }
            ArmInstruction::MUL(c, o) => {
                format!("{}{} {} r{}, r{}, r{}", self, c, o.s, o.rd, o.rm, o.rs)
            }
            ArmInstruction::SMULL(c, o) | ArmInstruction::SMLAL(c, o) |
            ArmInstruction::UMULL(c, o) | ArmInstruction::UMLAL(c, o) => {
                format!(
                    "{}{} {} r{}, r{}, r{}, r{}",
                    self, c, o.s, o.rd_hi, o.rd_lo, o.rm, o.rs
                )
            }
            ArmInstruction::B(c, o) | ArmInstruction::BL(c, o) => {
                format!("{}{} +{:#x}", self, c, o.offset)
            }
            ArmInstruction::BX(c, o) => {
                format!("{}{} r{}", self, c, o.rn)
            }
            ArmInstruction::SWP(c, o) | ArmInstruction::SWPB(c, o) => {
                format!("{}{} {} r{}, r{}, r{}", self, c, o.b, o.rd, o.rm, o.rn)
            }
            // TODO: Expand this
            ArmInstruction::LDM(c, _) | ArmInstruction::STM(c, _) => {
                // TODO: This is actually more complicated
                format!("{}{}", self, c)
            }
            // TODO: Expand this
            ArmInstruction::LDR(c, _) | ArmInstruction::STR(c, _) |
            ArmInstruction::CDP(c, _) | ArmInstruction::LDC(c, _) |
            ArmInstruction::STC(c, _) | ArmInstruction::MRC(c, _) |
            ArmInstruction::MCR(c, _) | ArmInstruction::MSR(c, _) |
            ArmInstruction::MRS(c, _) => {
                // TODO: This is actually more complicated
                format!("{}{}", self, c)
            }
            ArmInstruction::SWI(c) => {
                format!("{}{}", self, c)
            },
            ArmInstruction::Undef(_) => {
                format!("undefined")
            }
            _ => {
                format!("Nothing")
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
            l: (inst >> 24 & 0x1) == 0x1,
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

#[derive(Debug, PartialEq)]
pub struct PsrTransferOp {
    i: bool,
    p: bool,
    rd: u8,
    rm: u8,
    rotate: u8,
    imm: u8,
}

impl From<u32> for PsrTransferOp {
    fn from(inst: u32) -> Self {
        Self {
            i: (inst >> 25 & 1) == 1,
            p: (inst >> 22 & 1) == 1,
            rd: (inst >> 12 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
            rotate: (inst >> 8 & 0xf) as u8,
            imm: (inst & 0xff) as u8
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HalfwordDataOp {
    mode: AddressingMode3,
    p: bool,
    u: bool,
    w: bool,
    l: bool,
    s: bool,
    h: bool,
    rn: u8,
    rd: u8,
}

impl From<u32> for HalfwordDataOp {
    fn from(inst: u32) -> Self {
        let p = (inst >> 24 & 1) == 1;
        let byte_offset = ((inst & 0xff) | (inst >> 8 & 0xff)) as u8;
        let rm = (inst & 0xf) as u8;

        let mode = match (is_halfword_data_tfx_imm(inst), p) {
            (false, false) => AddressingMode3::PostIndexedReg { rm },
            (false, true) => AddressingMode3::PreIndexedReg { rm },
            (true, false) => AddressingMode3::PostIndexedImm { byte_offset },
            (true, true) => AddressingMode3::PreIndexedImm { byte_offset },
        };

        Self {
            p,
            u: (inst >> 23 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            s: (inst >> 6 & 1) == 1,
            h: (inst >> 5 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            mode,
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

pub fn is_mrs_op(inst: u32) -> bool {
    inst & 0x0fbf0fff == 0x010f0000
}

pub fn is_psr_transfer(inst: u32) -> bool {
    (inst & 0x0fbffff0 == 0x010f0000) ||
    (inst & 0x0fbf0fff == 0x010f0000) ||
    (inst & 0x0dbff000 == 0x0128f000)
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
        let op = ArmInstruction::from(inst);
        let op2 = ArmInstruction::B(
            Conditional::AL,
            BranchOp {
                l: false,
                offset: 0b11000,
            },
        );
        println!("{:?}", op);
        println!("{:?}", op2);
        assert_eq!(op, op2);
    }

    #[test]
    fn test_mrs_op_check() {
        let inst: u32 = 0xe14fc000;
        let is_mrs = is_mrs_op(inst);
        assert_eq!(is_mrs, true);
    }

    #[test]
    fn test_strb_decode() {
        let inst: u32 = 0xe5cc3301;
        let op = ArmInstruction::from(inst);
        let op2 = ArmInstruction::STR(Conditional::AL, SingleDataTfx {
            i: false,
            p: true,
            u: true,
            b: true,
            w: false,
            l: false,
            rn: 12,
            rd: 3,
            offset: 0x301,
        });
        assert_eq!(op, op2);
    }

    #[test]
    fn test_strh_decode() {
        let inst: u32 = 0xe08180b3;
        let op = ArmInstruction::from(inst);
        let op2 = ArmInstruction::STRH(Conditional::AL, HalfwordDataOp{
            p: false,
            u: true,
            w: false,
            l: false,
            h: true,
            s: false,
            rn: 1,
            rd: 8,
            mode: AddressingMode3::PostIndexedReg { rm: 3 },
        });
        assert_eq!(op, op2);
    }
}
