use crate::utils::Bitable;

pub(super) struct BgControl {
    pub bg_priority: u32,
    pub character_base_block: usize,
    pub mosaic: bool,
    pub pallete: bool,
    pub screen_base_block: usize,
    pub display_area_wraparound: bool,
    pub screen_size: u32,

}

impl From<u32> for BgControl {
    fn from(value: u32) -> Self {
        BgControl {
            bg_priority: value & 0b11,
            character_base_block: ((value >> 2) & 0b11) as usize,
            mosaic: value.bit_is_high(6),
            pallete: value.bit_is_high(7),
            screen_base_block: ((value >> 8) & 0x1f) as usize,
            display_area_wraparound: value.bit_is_high(13),
            screen_size: (value >> 14) & 0b11,
        }
    }
}

pub(super) struct BgOffset {
    x: u32,
    y: u32,
}

impl From<u32> for BgOffset {
    fn from(value: u32) -> Self {
        BgOffset {
            x: value & 0x1ff,
            y: (value >> 16) & 0x1ff,
        }
    }
}

pub(super) struct BgRotScale {
    pub ref_x: u32,
    pub ref_y: u32,
    pub dx: u32,
    pub dmx: u32,
    pub dy: u32,
    pub dmy: u32,
}

impl From<&[u32; 4]> for BgRotScale {
    fn from(value: &[u32; 4]) -> Self {
        BgRotScale {
            dx: value[0].halfword_at(0),
            dmx: value[0].halfword_at(16),
            dy: value[1].halfword_at(0),
            dmy: value[1].halfword_at(16),
            ref_x: value[2] & 0xfffffff,
            ref_y: value[3] & 0xfffffff,
        }
    }
}
