pub trait ArmCalculations {
    fn arm_add(self, rhs: u32) -> (u32, bool, bool);
    fn arm_sub(self, rhs: u32) -> (u32, bool, bool);
    fn arm_sub_carry(self, rhs: u32) -> (u32, bool, bool);
    fn arm_add_carry(self, rhs: u32) -> (u32, bool, bool);
}

/// res, c, v
impl ArmCalculations for u32 {
    fn arm_add(self, rhs: u32) -> (u32, bool, bool) {
        todo!()
    }

    fn arm_sub(self, rhs: u32) -> (u32, bool, bool) {
        todo!()
    }

    fn arm_sub_carry(self, rhs: u32) -> (u32, bool, bool) {
        todo!()
    }

    fn arm_add_carry(self, rhs: u32) -> (u32, bool, bool) {
        todo!()
    }
}
