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

#[derive(PartialEq)]
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
        CPU::new(0, 0, 0)
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
        write!(f, "GEN: {:?},", self.registers)?;
        write!(f, "FIQ: {:?},", self.fiq_banked_gen_regs)?;
        write!(f, "SVC: {:?},", self.svc_banked_regs)?;
        write!(f, "ABT: {:?},", self.abt_banked_regs)?;
        write!(f, "IRQ: {:?},", self.irq_banked_regs)?;
        write!(f, "Und: {:?},", self.und_banked_regs)?;
        write!(f, "psr: {:?},", self.psr)?;
        write!(f, "decode: {:?},", self.decode)?;
        write!(f, "addr: {:?},", self.inst_addr)?;
        write!(f, "cycles: {:?},", self.cycles)?;
        write!(f, "cpsr: {:08x}", self.cpsr)
    }
}

impl CPU {
    pub fn new(initial_pc: u32, initial_sp: u32, init_cycles: u32) -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, initial_sp, 0, initial_pc],
            fiq_banked_gen_regs: [0, 0, 0, 0, 0, initial_sp, 0],
            svc_banked_regs: [initial_sp, 0],
            abt_banked_regs: [initial_sp, 0],
            irq_banked_regs: [initial_sp, 0],
            und_banked_regs: [initial_sp, 0],
            psr: [0x1f,0,0,0,0,0],
            cpsr: 0x1f,
            decode: 0x0,
            inst_addr: 0x0,
            cycles: init_cycles,
        }
    }

    // Program Counter
    pub fn pc(&self) -> usize {
        self.registers[PC] as usize
    }

    pub fn instruction_address(&self) -> usize {
        self.inst_addr
    }

    pub fn add_cycles(&mut self, cycles: u32) {
        debug!("Current cycle count {}. Adding {} to cycle count", self.cycles, cycles);
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

        self.run_instruction(ram, inst, i_addr);
    }

    fn run_instruction(&mut self, ram: &mut SystemMemory, inst: u32, i_addr: usize) {
        let op = if !self.is_thumb_mode() {
            let cond = Conditional::from(inst);
            if !cond.should_run(self.cpsr) {
                debug!("Skipping: {:#08x}: {:#08x}", i_addr, inst);
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

mod test {
    use super::{CPU, SP, PC};
    use crate::SystemMemory;

    #[test]
    fn run_add_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 0, 0, 0, 12, 0, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ..CPU::default()
        };

        cpu.run_instruction(&mut ram, 0xe0844006, 0x0);

        let rhs = CPU {
            registers: [0, 0, 0, 0, 35, 0, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            cycles: 1,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_add_thumb_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [2, 8, 0, 0, 12, 3, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ..CPU::default()
        };
        cpu.update_thumb(true);

        cpu.run_instruction(&mut ram, 0x1909, 0x0);
        cpu.run_instruction(&mut ram, 0x4368, 0x0);

        let mut rhs = CPU {
            registers: [6, 20, 0, 0, 12, 3, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            cycles: 3,
            ..CPU::default()
        };
        rhs.update_thumb(true);
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_ldm_stm_instructions() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x100, 14, 15],
            ..CPU::default()
        };

        // r4,r5,r6,r7,r8,r9,r10,r11,lr
        cpu.run_instruction(&mut ram, 0xe92d4ff0, 0x0);

        assert_eq!(14, ram.read_word(0x0fc).unwrap());
        assert_eq!(11, ram.read_word(0x0f8).unwrap());
        assert_eq!(10, ram.read_word(0x0f4).unwrap());
        assert_eq!(9, ram.read_word(0x0f0).unwrap());
        assert_eq!(8, ram.read_word(0x0ec).unwrap());
        assert_eq!(7, ram.read_word(0x0e8).unwrap());
        assert_eq!(6, ram.read_word(0x0e4).unwrap());
        assert_eq!(5, ram.read_word(0x0e0).unwrap());
        assert_eq!(4, ram.read_word(0x0dc).unwrap());
        assert_eq!(10, cpu.cycles());

        cpu.registers = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0xdc, 255, 255];
        cpu.run_instruction(&mut ram, 0xe8bd4ff0, 0x0);

        let rhs = CPU {
            registers: [255, 255, 255, 255, 4, 5, 6, 7, 8, 9, 10, 11, 255, 256, 14, 255],
            cycles: 21,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_push_pop_instructions() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0x18, 15, 16],
            ..CPU::default()
        };
        cpu.update_thumb(true);

        //r4, r5, r7, lr
        cpu.run_instruction(&mut ram, 0xb5b0, 0x0);

        assert_eq!(15, ram.read_word(0x14).unwrap());
        assert_eq!(8, ram.read_word(0x10).unwrap());
        assert_eq!(6, ram.read_word(0x0c).unwrap());
        assert_eq!(5, ram.read_word(0x08).unwrap());
        assert_eq!(5, cpu.cycles());

        cpu.registers = [0; 16];
        cpu.registers[SP] = 0x08;
        cpu.run_instruction(&mut ram, 0xbcb0, 0x0);

        let mut rhs = CPU {
            registers: [0, 0, 0, 0, 5, 6, 0, 8, 0, 0, 0, 0, 0, 0x14, 0, 0],
            cycles: 10,
            ..CPU::default()
        };
        rhs.update_thumb(true);
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_ldm_stm_instructions_pak_ram() {
        let mut ram = SystemMemory::test_pak_ram();
        let mut cpu = CPU {
            registers: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x8000100, 14, 15],
            ..CPU::default()
        };

        //store: r4,r5,r6,r7,r8,r9,r10,r11,lr
        cpu.run_instruction(&mut ram, 0xe92d4ff0, 0x0);

        assert_eq!(0, ram.read_word(0x8000100).unwrap());
        assert_eq!(14, ram.read_word(0x80000fc).unwrap());
        assert_eq!(11, ram.read_word(0x80000f8).unwrap());
        assert_eq!(10, ram.read_word(0x80000f4).unwrap());
        assert_eq!(9, ram.read_word(0x80000f0).unwrap());
        assert_eq!(8, ram.read_word(0x80000ec).unwrap());
        assert_eq!(7, ram.read_word(0x80000e8).unwrap());
        assert_eq!(6, ram.read_word(0x80000e4).unwrap());
        assert_eq!(5, ram.read_word(0x80000e0).unwrap());
        assert_eq!(4, ram.read_word(0x80000dc).unwrap());
        assert_eq!(73, cpu.cycles());

        cpu.registers = [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0x80000dc, 255, 255];
        //load: r4,r5,r6,r7,r8,r9,r10,r11,lr
        cpu.run_instruction(&mut ram, 0xe8bd4ff0, 0x0);

        let rhs = CPU {
            registers: [255, 255, 255, 255, 4, 5, 6, 7, 8, 9, 10, 11, 255, 0x8000100, 14, 255],
            cycles: 147,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_push_pop_instructions_pak_ram() {
        let mut ram = SystemMemory::test_pak_ram();
        let mut cpu = CPU {
            registers: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0x8000018, 15, 16],
            ..CPU::default()
        };
        cpu.update_thumb(true);

        //push: r4, r5, r7, lr
        cpu.run_instruction(&mut ram, 0xb5b0, 0x0);

        assert_eq!(15, ram.read_word(0x8000014).unwrap());
        assert_eq!(8, ram.read_word(0x8000010).unwrap());
        assert_eq!(6, ram.read_word(0x800000c).unwrap());
        assert_eq!(5, ram.read_word(0x8000008).unwrap());
        assert_eq!(33, cpu.cycles());

        cpu.registers = [0; 16];
        cpu.registers[SP] = 0x8000008;
        //pop: r4, r5, r7
        cpu.run_instruction(&mut ram, 0xbcb0, 0x0);

        let mut rhs = CPU {
            registers: [0, 0, 0, 0, 5, 6, 0, 8, 0, 0, 0, 0, 0, 0x8000014, 0, 0],
            cycles: 59,
            ..CPU::default()
        };
        rhs.update_thumb(true);
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_pop_sp_with_pc() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0; 16],
            ..CPU::default()
        };
        cpu.registers[SP] = 0x10;
        cpu.update_thumb(true);
        let _ = ram.write_word(0x10, 2);
        let _ = ram.write_word(0x14, 3);
        let _ = ram.write_word(0x18, 4);

        //r4, r5, pc
        cpu.run_instruction(&mut ram, 0xbd30, 0x0);

        assert_eq!(cpu.registers[4], 2);
        assert_eq!(cpu.registers[5], 3);
        assert_eq!(cpu.registers[PC], 6);
        assert_eq!(cpu.cycles, 7);
    }

    #[test]
    fn check_cycles_thumb_ldrh() {
        let mut ram = SystemMemory::test_pak_ram();
        let mut cpu = CPU {
            registers: [0; 16],
            ..CPU::default()
        };
        cpu.registers[1] = 0x8000000;
        cpu.update_thumb(true);
        let _ = ram.write_word(0x8000000, 0xaaaaffff);

        //r4, r5, pc
        cpu.run_instruction(&mut ram, 0x8808, 0x0);

        assert_eq!(cpu.registers[0], 0xffff);
        assert_eq!(cpu.cycles, 7);
    }

    #[test]
    fn run_mov_with_reg_shift_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 0, 0, 0, 12, 0, 1, 0, 0, 8, 0, 0, 0, 0, 0, 0],
            ..CPU::default()
        };

        cpu.run_instruction(&mut ram, 0xe1a06916, 0x0);

        let rhs = CPU {
            registers: [0, 0, 0, 0, 12, 0, 256, 0, 0, 8, 0, 0, 0, 0, 0, 0],
            cycles: 2,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_mov_with_reg_shift_0_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 0, 0, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ..CPU::default()
        };

        cpu.run_instruction(&mut ram, 0xe1a06916, 0x0);

        let rhs = CPU {
            registers: [0, 0, 0, 0, 12, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            cycles: 2,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_mov_with_reg_imm_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 0, 2, 0, 12, 0, 1, 0, 0, 8, 0, 0, 0, 0, 0, 0],
            ..CPU::default()
        };

        cpu.run_instruction(&mut ram, 0xe1a0a0a2, 0x0);

        let rhs = CPU {
            registers: [0, 0, 2, 0, 12, 0, 1, 0, 0, 8, 1, 0, 0, 0, 0, 0],
            cycles: 1,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_check_mov_cpsr_with_shift_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 0, 2, 0, 0, 0, 1, 0, 0, 8, 0, 3, 0, 0, 0, 0],
            ..CPU::default()
        };

        cpu.run_instruction(&mut ram, 0xe1b0c43b, 0x0);

        let rhs = CPU {
            registers: [0, 0, 2, 0, 0, 0, 1, 0, 0, 8, 0, 3, 3, 0, 0, 0],
            cycles: 2,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }

    #[test]
    fn run_thumb_cmp_instruction() {
        let mut ram = SystemMemory::test();
        let mut cpu = CPU {
            registers: [0, 0, 0, 0, 0, 0, 0xffffff55, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            ..CPU::default()
        };
        cpu.update_thumb(true);

        cpu.run_instruction(&mut ram, 0x2e00, 0x0);

        let rhs = CPU {
            registers: [0, 0, 0, 0, 0, 0, 0xffffff55, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            cycles: 1,
            cpsr: 0xa000003f,
            ..CPU::default()
        };
        assert_eq!(cpu, rhs);
    }
}
