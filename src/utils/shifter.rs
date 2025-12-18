use super::Bitable;
use crate::gba::cpu::Cpu;

pub trait CpuShifter {
    fn shl_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn shr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn asr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn ror_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn rrx_with_carry(&self, lhs: u32) -> (u32, bool);
}

impl CpuShifter for Cpu {
    fn shl_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool) {
        match rhs {
            0 => (lhs, self.c_status()),
            1..=31 => (lhs << rhs, lhs.bit_is_high(32 - rhs)),
            32 => (0, lhs.bit_is_high(0)),
            _ => (0, false),
        }
    }

    fn shr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool) {
        match rhs {
            0 => (lhs, self.c_status()),
            1..=31 => (lhs >> rhs, lhs.bit_is_high(rhs - 1)),
            32 => (0, lhs.bit_is_high(31)),
            _ => (0, false),
        }
    }

    fn asr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool) {
        let lhs = lhs as i32;
        if rhs == 0 {
            (lhs as u32, self.c_status())
        } else if rhs > 31 {
            ((lhs >> 31) as u32, lhs.bit_is_high(31))
        } else {
            ((lhs >> rhs) as u32, lhs.bit_is_high(rhs - 1))
        }
    }

    fn ror_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool) {
        if rhs == 0 {
            (lhs, self.c_status())
        } else {
            let rhs = rhs % 32;
            if rhs == 0 {
                (lhs, lhs.bit_is_high(31))
            } else {
                (lhs.rotate_right(rhs), lhs.bit_is_high(rhs - 1))
            }
        }
    }

    fn rrx_with_carry(&self, lhs: u32) -> (u32, bool) {
        let c_in = if self.c_status() { 0x80000000 } else { 0 };
        ((lhs >> 31) | c_in, (lhs & 1) == 0b1)
    }
}

mod test {
    #![allow(unused)]
    use super::{Cpu, CpuShifter};

    #[test]
    fn ror_32_1() {
        let cpu = Cpu::default();
        let (res, carry) = cpu.ror_with_carry(0xa2cef820, 32);
        assert_eq!(res, 0xa2cef820);
        assert!(carry);
    }

    #[test]
    fn lsl_1() {
        let mut cpu = Cpu::default();
        cpu.set_v_status(true);
        let (res, carry) = cpu.shl_with_carry(0xbfbfc0cf, 0xb);
        assert_eq!(res, 0xfe067800);
        assert!(carry);
    }

    #[test]
    fn lsr_1() {
        let mut cpu = Cpu::default();
        cpu.set_v_status(true);
        let (res, carry) = cpu.shr_with_carry(0xb220b2e9, 0x21);
        assert_eq!(res, 0x0);
        assert!(!carry);
    }
}
