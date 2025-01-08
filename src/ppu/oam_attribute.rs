use crate::{gba::system::SystemMemory, utils::Bitable};

const ROT_SCALE_FLAG: u32 = 0x100;
const OBJECT_FLAG: u32 = 0x200;
const BASE_OBJ_PALETTE: usize = 0x200;

pub fn is_oam_entry_enabled(value: &[u32]) -> bool {
    (value[0] >> 8 & 0b11) != 0b10
}

#[derive(Debug)]
pub struct OamAttribute {
    pub y_coord: u32,
    pub x_coord: u32,
    pub is_rot_scale: bool,
    pub object_flag: ObjectFlag,
    pub obj_mode: u32,
    pub obj_mosaic: bool,
    pub colors: bool,
    pub obj_shape: Shape,
    pub character_name: u32,
    pub priority: u32,
    pub palette: usize,
    pub transformation: Transformation,
}

impl OamAttribute {
    pub fn get_palette<'a>(&self, ram: &'a SystemMemory) -> &'a [u32] {
        let palette_ram = ram.get_palette_ram_slice();
        if self.colors {
            let start = BASE_OBJ_PALETTE + (16 * self.palette);
            &palette_ram[start..start+16]
        } else {
            &palette_ram[BASE_OBJ_PALETTE..BASE_OBJ_PALETTE+256]
        }
    }
}

#[derive(Debug)]
pub struct Shape {
    x: u32,
    y: u32
}

#[derive(Debug)]
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

#[derive(Debug)]
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
        let obj_shape = (value[0] >> 14) & 0b11;
        let obj_size = (value[0] >> 29) & 0b11;
        let secondary_size = [8, 8, 16, 32];

        let base_size = 8 * (obj_size + 1);
        let shape = match obj_shape {
            0 => Shape { x: base_size, y: base_size },
            1 => Shape { x: base_size, y: secondary_size[obj_size as usize] },
            2 => Shape { x: secondary_size[obj_size as usize], y: base_size },
            _ => panic!()
        };

        Self {
            y_coord: value[0] & 0xff,
            x_coord: value[1] & 0x1ff,
            is_rot_scale: value[0].bit_is_high(8),
            object_flag: ObjectFlag::from(value),
            obj_shape: shape,
            obj_mode: (value[0] >> 10) & 0b11,
            obj_mosaic: value[0].bit_is_high(12),
            colors: value[0].bit_is_high(13),
            transformation: Transformation::from(value),
            character_name: value[1] & 0x3ff,
            priority: (value[1] >> 10) & 0b11,
            palette: ((value[1] >> 12) & 0b111) as usize,
        }
    }
}

// TODO: Back port this to BgRotScale?
#[derive(Debug)]
pub struct RotationScaleParameter {
    pub dx: u32,
    pub dmx: u32,
    pub dy: u32,
    pub dmy: u32,
}

pub struct RotationScaleParameterBuilder {
    parameters: Vec<u32>
}

impl RotationScaleParameterBuilder {
    pub fn new() -> Self {
        Self {
            parameters: Vec::new()
        }
    }

    pub fn add_parameter(&mut self, param: u32) {
        self.parameters.push((param >> 16) & 0xffff);
    }

    pub fn build(&self) -> Vec<RotationScaleParameter> {
        let mut res = Vec::new();

        for x in self.parameters.chunks(4) {
            res.push(RotationScaleParameter {
                dx: x[0],
                dmx: x[1],
                dy: x[2],
                dmy: x[3],
            });
        }

        res
    }
}
