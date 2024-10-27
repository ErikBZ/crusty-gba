use super::bit_is_one_at;

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
        } else {
            let x = if rhs > 31 { 31 } else { rhs };
            (((self as i32) >> x) as u32, bit_is_one_at(self, x - 1))
        }
    }

    fn ror_with_carry(self, rhs: u32) -> (u32, bool) {
        let carry = if rhs == 0 {
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

}
