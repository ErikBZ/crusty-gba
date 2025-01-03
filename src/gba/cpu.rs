use core::fmt;
use super::arm::decode_as_arm;
use super::thumb::decode_as_thumb;
use super::{is_signed, Conditional, CPSR_C, CPSR_N, CPSR_T, CPSR_V, CPSR_Z};
use super::system::SystemMemory;
use tracing::{debug, trace};

pub const PC: usize = 15;
pub const LR: usize = 14;
pub const SP: usize = 13;

#[derive(Debug, PartialEq, Eq)]
pub enum CpuMode {
    System,
    User,
    FIQ,
    Supervisor,
    Abort,
    IRQ,
    Undefined
}

impl From<u32> for CpuMode {
    fn from(value: u32) -> Self {
        match value & 0x1f {
            0b10000 => CpuMode::User,
            0b10001 => CpuMode::FIQ,
            0b10010 => CpuMode::IRQ,
            0b10011 => CpuMode::Supervisor,
            0b10111 => CpuMode::Abort,
            0b11011 => CpuMode::Undefined,
            0b11111 => CpuMode::System,
            _ => unreachable!(),
        }
    }
}

pub struct CPU {
    registers: [u32; 16],
    // NOTE: General use banked regs, r8-r12
    fiq_banked_gen_regs: [u32; 7],
    // NOTE: Banked regs r13, r14 for all alt modes
    svc_banked_regs: [u32; 2],
    abt_banked_regs: [u32; 2],
    irq_banked_regs: [u32; 2],
    und_banked_regs: [u32; 2],
    pub cpsr: u32,
    psr: [u32; 6],
    pub decode: u32,
    pub inst_addr: usize,
    cycles: u32,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03007f00, 0, 0x68],
            fiq_banked_gen_regs: [0, 0, 0, 0, 0, 0x03007f00, 0],
            svc_banked_regs: [0x03007f00, 0],
            abt_banked_regs: [0x03007f00, 0],
            irq_banked_regs: [0x03007f00, 0],
            und_banked_regs: [0x03007f00, 0],
            psr: [0x1f,0,0,0,0,0],
            cpsr: 0x1f,
            decode: 0x0,
            inst_addr: 0x0,
            cycles: 2,
        }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in (0..16).step_by(4) {
            write!(f, "r{}\t{:#08x}\t", i, self.get_register(i))?;
            write!(f, "r{}\t{:#08x}\t", i + 1, self.get_register(i + 1))?;
            write!(f, "r{}\t{:#08x}\t", i + 2, self.get_register(i + 2))?;
            write!(f, "r{}\t{:#08x}\n", i + 3, self.get_register(i + 3))?;
        }
        write!(f, "cpsr: {:#8x}, cycles: {}, instruction address: {:#08x}\n", self.cpsr, self.cycles, self.instruction_address())?;

        let cond = Conditional::from(self.decode);
        if self.is_thumb_mode() {
            let op = decode_as_thumb(self.decode);
            write!(f, "{:#04x} {:?} {:?}", self.decode, cond, op)
        } else {
            let op = decode_as_arm(self.decode);
            write!(f, "{:#08x} {:?} {:?}", self.decode, cond, op)
        }
    }
}

// To speed up debugging we'll be printing just the `registers` field
impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..16 {
            write!(f, "r{}: {:#08x}, ", i, self.get_register(i))?;
        }
        write!(f, "cpsr: {:08x}", self.cpsr)
    }
}

impl CPU {
    // Program Counter
    pub fn pc(&self) -> usize {
        self.registers[PC] as usize
    }

    pub fn instruction_address(&self) -> usize {
        self.inst_addr
    }

    pub fn add_cycles(&mut self, cycles: u32) {
        trace!("Current cycle count {}. Adding {} to cycle count", self.cycles, cycles);
        self.cycles = self.cycles.wrapping_add(cycles);
    }

    pub fn cycles(&self) -> u32 {
        self.cycles
    }

    // TODO: Do reverse for set_register
    pub fn get_register(&self, rn: usize) -> u32 {
        let mode = CpuMode::from(self.cpsr);
        if rn == 15 || (rn < 13 && !((mode == CpuMode::FIQ) && rn > 8)) {
            return self.registers[rn];
        }

        match mode {
            CpuMode::FIQ => self.fiq_banked_gen_regs[rn - 8],
            CpuMode::Supervisor => self.svc_banked_regs[rn - 13],
            CpuMode::IRQ => self.irq_banked_regs[rn - 13],
            CpuMode::Abort => self.abt_banked_regs[rn - 13],
            CpuMode::Undefined => self.und_banked_regs[rn - 13],
            CpuMode::User | CpuMode::System => self.registers[rn],
        }
    }

    pub fn set_register(&mut self, rn: usize, value: u32) {
        let mode = CpuMode::from(self.cpsr);
        if rn == 15 || (rn < 13 && !((mode == CpuMode::FIQ) && rn > 8)) {
            return self.registers[rn] = value;
        }

        match mode {
            CpuMode::FIQ => self.fiq_banked_gen_regs[rn - 8] = value,
            CpuMode::Supervisor => self.svc_banked_regs[rn - 13] = value,
            CpuMode::Abort => self.abt_banked_regs[rn - 13] = value,
            CpuMode::IRQ => self.irq_banked_regs[rn - 13] = value,
            CpuMode::Undefined => self.und_banked_regs[rn - 13] = value,
            CpuMode::User | CpuMode::System => self.registers[rn] = value,
        }
    }

    // Note: will return the CPSR when mode is sys or user, and
    // corresponding spsr for other modes
    pub fn get_psr(&self) -> u32 {
        let mode = CpuMode::from(self.cpsr);
        match mode {
            CpuMode::User | CpuMode::System => self.cpsr,
            CpuMode::FIQ => self.psr[0],
            CpuMode::Supervisor => self.psr[1],
            CpuMode::IRQ => self.psr[2],
            CpuMode::Abort => self.psr[3],
            CpuMode::Undefined => self.psr[4],
        }
    }

    pub fn set_psr(&mut self, value: u32) {
        let mode = CpuMode::from(self.cpsr);
        match mode {
            CpuMode::User | CpuMode::System => println!("Can't set SPSR in User and System mode"),
            CpuMode::FIQ => self.psr[0] = value,
            CpuMode::Supervisor => self.psr[1] = value,
            CpuMode::IRQ => self.psr[2] = value,
            CpuMode::Abort => self.psr[3] = value,
            CpuMode::Undefined => self.psr[4] = value,
        }
    }

    pub fn update_cpsr(&mut self, res: u32, v: bool, c: bool) {
        let zero = if res == 0 {
            CPSR_Z
        } else {
            0
        };

        let neg = if is_signed(res) {
            CPSR_N
        } else {
            0
        };

        // Over is set when POS + POS = neg, NEG + NEG = pos
        // or POS - NEG = NEG, NEG - POS = POS
        let over = if v {
            CPSR_V
        } else {
            0
        };

        let carry = if c {
            CPSR_C
        } else {
            0
        };

        self.cpsr &= 0x0fffffff;
        // self.cpsr |= CPSR_C & (res >> 2);
        self.cpsr |= zero;
        self.cpsr |= neg;
        self.cpsr |= over;
        self.cpsr |= carry;
    }

    pub fn update_thumb(&mut self, is_thumb: bool) {
        if is_thumb {
            self.cpsr |= CPSR_T;
        } else {
            self.cpsr &= !CPSR_T;
        }
    }

    pub fn is_thumb_mode(&self) -> bool {
        self.cpsr & CPSR_T == CPSR_T
    }

    pub fn tick(&mut self, ram: &mut SystemMemory) {
        let inst = self.decode;
        let i_addr = self.inst_addr;
        self.inst_addr = self.pc();
        let next_inst = if self.is_thumb_mode() {
            ram.read_halfword(self.pc())
        } else {
            ram.read_word(self.pc())
        };

        self.decode = match next_inst {
            Ok(i) => i,
            Err(e) => {
                println!("{}", e);
                0
            }
        };

        // NOTE: I think this has to happen after run
        // that's why the reg is always 8 ahead, and not just 4 ahead
        self.registers[PC] += if !self.is_thumb_mode() {
            4
        } else {
            2
        };

        let op = if !self.is_thumb_mode() {
            let cond = Conditional::from(inst);
            if !cond.should_run(self.cpsr) {
                self.add_cycles(1);
                return;
            }
            decode_as_arm(inst)
        } else {
            decode_as_thumb(inst)
        };
        debug!("{:#08x}: {:#08x} - {:?}", i_addr, inst, op);

        op.run(self, ram);
    }

    pub fn tick_for_cycles(&mut self, ram: &mut SystemMemory, num_of_cycles: u32) {
        let old_cycles = self.cycles;
        while self.cycles - old_cycles < num_of_cycles {
            self.tick(ram);
        }
    }
}
