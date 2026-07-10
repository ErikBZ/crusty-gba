use tracing::trace;
pub trait ArmCalculations {
    fn arm_add(self, rhs: u32) -> (u32, bool, bool);
    fn arm_sub(self, rhs: u32) -> (u32, bool, bool);
    fn arm_sub_carry(self, rhs: u32, carry_in: bool) -> (u32, bool, bool);
    fn arm_add_carry(self, rhs: u32, carry_in: bool) -> (u32, bool, bool);
}

/// res, c, v
impl ArmCalculations for u32 {
    // NOTE: Comeback to this and simplify it like arm_sub
    /// res, c, v
    fn arm_add(self, rhs: u32) -> (u32, bool, bool) {
        let res = self.wrapping_add(rhs);
        let carry = ((self & rhs) | ((self ^ rhs) & !res)) >= 0x80000000;
        let overflow = ((self ^ res) & (rhs ^ res)) & 0x80000000 != 0;
        (res, carry, overflow)
    }

    /// res, c, v
    fn arm_sub(self, rhs: u32) -> (u32, bool, bool) {
        let (intermediate, i_carry) = self.overflowing_add(!rhs);
        let (res, carry, _) = intermediate.arm_add(1);

        // Checks for overflow when subtracting
        let v = ((self^ rhs) & (self ^ res)) >= 0x80000000;
        (res, carry | i_carry, v)
    }

    /// res, c, v
    fn arm_add_carry(self, rhs: u32, carry_in: bool) -> (u32, bool, bool) {
        let carry_in = if carry_in {1} else {0};
        let res = self.wrapping_add(rhs).wrapping_add(carry_in);
        let carry = ((self & rhs) | ((self ^ rhs) & !res)) >= 0x80000000;
        let overflow = ((self ^ res) & (rhs ^ res)) & 0x80000000 != 0;
        (res, carry, overflow)
    }

    /// res, c, v
    fn arm_sub_carry(self, rhs: u32, carry_in: bool) -> (u32, bool, bool) {
        let (intermediate, i_carry) = self.overflowing_add(!rhs);
        let carry = if carry_in {1} else {0};
        let (res, carry, _) = intermediate.arm_add(carry);

        // Checks for overflow when subtracting
        let v = ((self^ rhs) & (self ^ res)) >= 0x80000000;
        (res, carry | i_carry, v)
    }
}

mod test {
    #![allow(unused)]
    use super::ArmCalculations;

    #[test]
    fn test_arm_subtract_1() {
        let lhs: u32 = 0x8173d9e8;
        let rhs: u32 = 0x2b6e0a2c;

        let (res, c, v) = lhs.arm_sub(rhs);
        let n = res >= 0x80000000;
        let z = res == 0;

        assert!(c);
        assert!(v);
        assert!(!z);
        assert!(!n);
    }

    // Proving that we can use the u32's instead of converting to bools
    #[test]
    fn test_arm_subtract_2() {
        let lhs: u32 = 0x8173d9e8;
        let rhs: u32 = 0x2b6e0a2c;

        let (intermediate, i_carry) = lhs.overflowing_add(!rhs);
        let (res, carry, _) = intermediate.arm_add(1);
        let lhs_sign = lhs >= 0x80000000;
        let rhs_sign = rhs >= 0x80000000;
        let res_sign = res >= 0x80000000;

        let v = (lhs_sign ^ rhs_sign) == (rhs_sign == res_sign);
        let v_other = ((lhs ^ rhs) & (lhs ^ res)) >= 0x80000000;
        assert_eq!(v, v_other)
    }

    #[test]
    fn test_arm_add_1() {
        let lhs: u32 = 0xcd717a79;
        let rhs: u32 = 0xba4eb81e;

        let (res, c, v) = lhs.arm_add(rhs);
        assert_eq!(res, 0x87c03297);
    }
}
