use crate::utils::Bitable;

const ROT_SCALE_FLAG: u32 = 0x100;
const OBJECT_FLAG: u32 = 0x200;

pub fn is_oam_entry_enabled(value: &[u32]) -> bool {
    (value[0] >> 8 & 0b11) != 0b10
}

pub struct OamAttribute {
    pub y_coord: u32,
    pub x_coord: u32,
    pub is_rot_scale: bool,
    pub object_flag: ObjectFlag,
    pub obj_mode: u32,
    pub obj_mosaic: bool,
    pub colors: bool,
    pub obj_shape: u32,
    pub obj_size: u32,
    pub character_name: u32,
    pub priority: u32,
    pub palette: usize,
    pub transformation: Transformation,
}

pub enum ObjectFlag {
    DoublSize(bool),
    Disbale(bool)
}

impl From<&[u32]> for ObjectFlag {
    fn from(value: &[u32]) -> Self {
        let rot = value[0] & ROT_SCALE_FLAG == ROT_SCALE_FLAG;
        let val = value[0] & OBJECT_FLAG == OBJECT_FLAG;
        if rot {
            Self::DoublSize(val)
        } else {
            Self::Disbale(val)
        }
    }
}

pub enum Transformation {
    RotScale { idx: usize },
    Flip { horizontal: bool, veritical: bool }
}

impl From<&[u32]> for Transformation {
    fn from(value: &[u32]) -> Self {
        let rot = value[0] & ROT_SCALE_FLAG == ROT_SCALE_FLAG;
        if rot {
            let rot_scale_param = (value[0] >> 24) & 0x1f;
            Transformation::RotScale {idx: rot_scale_param as usize}
        } else {
            let horizontal = (value[0] >> 27) == 0x1;
            let veritical = (value[0] >> 28) == 0x1;
            Transformation::Flip { horizontal, veritical }
        }
    }
}

impl From<&[u32]> for OamAttribute {
    fn from(value: &[u32]) -> Self {
        Self {
            y_coord: value[0] & 0xff,
            x_coord: value[1] & 0x1ff,
            is_rot_scale: value[0].bit_is_high(8),
            object_flag: ObjectFlag::from(value),
            obj_mode: (value[0] >> 10) & 0b11,
            obj_mosaic: value[0].bit_is_high(12),
            colors: value[0].bit_is_high(13),
            obj_shape: (value[0] >> 14) & 0b11,
            transformation: Transformation::from(value),
            obj_size: (value[0] >> 29) & 0b11,
            character_name: value[1] & 0x3ff,
            priority: (value[1] >> 10) & 0b11,
            palette: ((value[1] >> 12) & 0b111) as usize,
        }
    }
}
