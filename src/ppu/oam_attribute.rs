use tracing::info;

use crate::{gba::system::SystemMemory, utils::{Bitable, BittableColor}};

const ROT_SCALE_FLAG: u32 = 0x100;
const OBJECT_FLAG: u32 = 0x200;
const BASE_OBJ_PALETTE: usize = 0x200 / 4;

pub fn is_oam_entry_enabled(value: &[u32]) -> bool {
    (value[0] >> 8 & 0b11) != 0b10
}

pub fn get_palettes(ram: &SystemMemory) -> Colors {
    let palette_ram = ram.get_palette_ram_slice();
    Colors::from(palette_ram)
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
    pub palette_idx: usize,
    pub transformation: Transformation,
}

#[derive(Debug)]
pub struct Shape {
    pub w: u32,
    pub h: u32
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
        let obj_size = (value[0] >> 30) & 0b11;
        let secondary_size = [8, 8, 16, 32];

        let base_size = 8 * (1 << obj_size);
        let shape = match obj_shape {
            0 => Shape { w: base_size, h: base_size },
            1 => Shape { w: base_size, h: secondary_size[obj_size as usize] },
            2 => Shape { w: secondary_size[obj_size as usize], h: base_size },
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
            palette_idx: ((value[1] >> 12) & 0b111) as usize,
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

// I could save everything into a palette and when i need to grab
// a color as if it's a 256/1 through a mapping function
pub struct Colors {
    palettes: Vec<Palette>,
    colors: Vec<(u8,u8,u8)>,
}

pub struct Palette {
    colors: Vec<(u8, u8, u8)>
}

impl Colors {
    pub fn get_palette(&self, palette_id: usize) -> &Vec<(u8, u8, u8)> {
        &self.palettes[palette_id].colors
    }

    pub fn num_of_palettes(&self) -> usize {
        self.palettes.len()
    }
}

impl From<&[u32]> for Colors {
    // TODO: This will need some heavy refactors
    fn from(value: &[u32]) -> Self {
        let obj_palette = &value[BASE_OBJ_PALETTE..BASE_OBJ_PALETTE + (512 / 4)];
        let mut palettes: Vec<Palette> = Vec::new();
        let mut colors: Vec<(u8, u8, u8)> = Vec::new();

        for x in obj_palette.chunks(8) {
            let mut pal_colors = Vec::new();

            for i in x {
                let (c1, c2) = i.to_8bit_color();
                pal_colors.push(c1);
                pal_colors.push(c2);
                colors.push(c1);
                colors.push(c2);
            }

            palettes.push(Palette {colors: pal_colors});
        }

        Self {
            palettes,
            colors
        }
    }
}
