pub mod shifter;
pub mod io_registers;

// Returns true if the bit at x is 1
/// Use u32.bit_is_high instead
#[deprecated]
pub fn bit_is_one_at(num: u32, x: u32) -> bool {
    if x > 31 {
        panic!()
    }

    (num >> x) & 1 == 1
}

pub trait Bitable {
    fn bit_is_high(&self, x: u32) -> bool;
    fn half_byte_at(&self, x: u32) -> u32;
    fn byte_at(&self, x: u32) -> u32;
    fn halfword_at(&self, x: u32) -> u32;
}

pub trait BittableColor {
    fn to_8bit_color(&self) -> ((u8, u8, u8), (u8, u8, u8));
}

impl Bitable for i32 {
    fn bit_is_high(&self, x: u32) -> bool {
        if x > 31 {
            panic!()
        } else {
            (self >> x) & 1 == 1
        }
    }
    fn byte_at(&self, _x: u32) -> u32 {
        todo!()
    }
    fn halfword_at(&self, _x: u32) -> u32 {
        todo!()
    }
    fn half_byte_at(&self, _x: u32) -> u32 {
        todo!()
    }
}

impl Bitable for u64 {
    fn bit_is_high(&self, x: u32) -> bool {
        // in release this should be false
        if x > 63 {
            panic!()
        } else {
            (self >> x) & 1 == 1
        }
    }

    fn half_byte_at(&self, x: u32) -> u32 {
        if x > 63 {
            panic!()
        } else {
            ((self >> x) & 0xf) as u32
        }
    }

    fn byte_at(&self, _x: u32) -> u32 {
        todo!()
    }

    fn halfword_at(&self, _x: u32) -> u32 {
        todo!()
    }
}

impl Bitable for u32 {
    fn bit_is_high(&self, x: u32) -> bool {
        // in release this should be false
        if x > 31 {
            panic!()
        } else {
            (self >> x) & 1 == 1
        }
    }

    fn half_byte_at(&self, x: u32) -> u32 {
        guard_shift(*self, x) & 0xf
    }

    fn byte_at(&self, x: u32) -> u32 {
        guard_shift(*self, x) & 0xff
    }

    fn halfword_at(&self, x: u32) -> u32 {
        guard_shift(*self, x) & 0xffff
    }
}

fn guard_shift(val: u32, x: u32) -> u32 {
    if x > 31 {
        panic!()
    }
    val >> x
}

impl BittableColor for u32 {
    // TODO: This is gross, find better way
    fn to_8bit_color(&self) -> ((u8, u8, u8), (u8, u8, u8)) {
        (
            (
                (self.byte_at(0) * 4) as u8,
                (self.byte_at(5) * 4) as u8,
                (self.byte_at(10) * 4) as u8,
            ),
            (
                (self.byte_at(16) * 4) as u8,
                (self.byte_at(21) * 4) as u8,
                (self.byte_at(26) * 4) as u8,
            )
        )
    }
}
