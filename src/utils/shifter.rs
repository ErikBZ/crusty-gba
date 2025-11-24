use std::ops::Shr;

use crate::gba::cpu::CPU;

use super::bit_is_one_at;
use super::Bitable;

pub trait CpuShifter {
    fn shl_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn shr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn asr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn ror_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool);
    fn rrx_with_carry(&self, lhs: u32) -> (u32, bool);
}

impl CpuShifter for CPU {
    fn shl_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool) {
        if rhs == 0 {
            (lhs, self.c_status())
        } else if rhs > 31 {
            (lhs << 31, lhs.bit_is_high(0))
        } else {
            (lhs << rhs, lhs.bit_is_high(32 - rhs))
        }
    }

    fn shr_with_carry(&self, lhs: u32, rhs: u32) -> (u32, bool) {
        if rhs == 0 {
            (lhs, self.c_status())
        } else if rhs > 31 {
            (0, lhs.bit_is_high(31))
        } else {
            (lhs >> rhs, lhs.bit_is_high(rhs - 1))
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
        let rhs = rhs % 32;
        if rhs == 0 {
            (lhs, false)
        } else {
            (lhs.rotate_right(rhs), lhs.bit_is_high(rhs - 1))
        }
    }

    fn rrx_with_carry(&self, lhs: u32) -> (u32, bool) {
        let c_in = if self.c_status() {0x80000000} else {0};
        ((lhs >> 31) | c_in, (lhs & 1) == 0b1)
    }
}

#[deprecated]
pub trait ShiftWithCarry {
    fn shl_with_carry(self, rhs: u32) -> (u32, bool);
    fn shr_with_carry(self, rhs: u32) -> (u32, bool);
    fn asr_with_carry(self, rhs: u32) -> (u32, bool);
    fn ror_with_carry(self, rhs: u32) -> (u32, bool);
    fn rrx_with_carry(self, c_in: bool) -> (u32, bool);
}

impl ShiftWithCarry for u32 {
    fn shl_with_carry(self, rhs: u32) -> (u32, bool) {
        let carry = if rhs > 32 {
            false
        } else {
            bit_is_one_at(self, 32 - rhs)
        };

        let res = if rhs > 31 {
            0
        } else {
            self << rhs
        };

        (res, carry)
    }

    fn shr_with_carry(self, rhs: u32) -> (u32, bool) {
        let carry = if rhs > 32 || rhs < 1 {
            false
        } else {
            bit_is_one_at(self, rhs - 1)
        };

        let res = if rhs > 31 {
            0
        } else {
            self >> rhs
        };

        (res, carry)
}

    fn asr_with_carry(self, rhs: u32) -> (u32, bool) {
        if rhs == 0 {
            (self, false)
        } else if rhs > 31 {
            let res = (self as i32) >> 31;
            (res as u32, bit_is_one_at(self, 31))
        } else {
            let res = (self as i32) >> rhs;
            (res as u32, bit_is_one_at(self, rhs - 1))
        }
    }

    fn ror_with_carry(self, rhs: u32) -> (u32, bool) {
        let carry = if rhs % 32 == 0 {
            false
        } else {
            bit_is_one_at(self, (rhs % 32) - 1)
        };

        (self.rotate_right(rhs), carry)
    }

    fn rrx_with_carry(self, c_in: bool) -> (u32, bool) {
        let c_out = self & 1 == 1;
        let c_value = if c_in { 0x80000000 } else { 0 };
        ((self >> 1) | c_value, c_out)
    }
}

mod test {
    use super::*;

    #[test]
    fn ror_32_1() {
        let (res, carry) = 0xa2cef820.ror_with_carry(32);
        assert_eq!(res, 0xa2cef820);
        assert!(!carry);
    }

    #[test]
    fn ror_32_2() {
        let (res, carry) = 0x08dbcc9c.ror_with_carry(32);
        assert_eq!(res, 0x08dbcc9c);
        assert!(!carry);
    }

    #[test]
    fn asr_number_by_0() {
        let (res, carry) = 0x08dbcc9c.ror_with_carry(32);
        assert_eq!(res, 0x08dbcc9c);
        assert!(!carry);
    }
}
