mod disp_control;
mod bg_control;
mod window_control;
mod color_effect;
mod oam_attribute;

use bg_control::{bg_control0, bg_control1, bg_control2, bg_control3, BgControl};
use oam_attribute::{get_palettes, OamAttribute, RotationScaleParameter, RotationScaleParameterBuilder};
use tracing::{warn, trace, info, debug, error};
use crate::{gba::system::MemoryError, utils::Bitable, SystemMemory};
use disp_control::{display_control, DisplayControl};
use crate::utils::io_registers::{DISP_STAT, V_COUNT};
// Base off of https://github.com/tuzz/game-loop 

const V_BLANK_FLAG: u32 = 0b00000001;
const H_BLANK_FLAG: u32 = 0b00000010;
const V_COUNTER_FLAG: u32 = 0b00000100;

const BASE_OAM: u32 = 0x6010000;
const HEIGHT: usize = 160;
const WIDTH: usize = 240;

fn set_bit_high(ram: &mut SystemMemory, addr: usize, flag: u32) {
    let io_ram = ram.get_io_ram();
    let idx = (addr >> 2) & 0xffff;
    io_ram[idx] = io_ram[idx] | flag;
}

fn set_bit_low(ram: &mut SystemMemory, addr: usize, flag: u32) {
    let io_ram = ram.get_io_ram();
    let idx = (addr >> 2) & 0xffff;
    io_ram[idx] = io_ram[idx] & !flag;
}

pub struct PPU {
    old_cycle: u32,
    h_count: u32,
    v_count: u32,
    // rename to frame_counter?
    frame: u32,
    next_frame: Vec<u8>,
}

impl Default for PPU {
    fn default() -> Self {
        PPU {
            old_cycle: 0,
            h_count: 0,
            v_count: 0,
            frame: 0,
            next_frame: vec![255; HEIGHT * WIDTH * 4]
        }
    }
}

impl PPU {
    pub fn tick(&mut self, cycle: u32, ram: &mut SystemMemory) -> bool {
        if cycle >> 2 == self.old_cycle {
            return false;
        }
        let delta_cycle = (cycle >> 2) - self.old_cycle;
        self.old_cycle = cycle >> 2;

        match self.update_io_registers(delta_cycle, ram) {
            Ok(b) => b,
            Err(e) => {
                warn!("{}", e);
                false
            }
        }
    }

    fn update_io_registers(&mut self, d_cycle: u32, ram: &mut SystemMemory) -> Result<bool, MemoryError> {
        let new_v = self.update_h_count(d_cycle, ram)?;
        if new_v {
            self.update_v_count(ram)
        } else {
            Ok(false)
        }
    }

    fn update_h_count(&mut self, d_cycle: u32, ram: &mut SystemMemory) -> Result<bool, MemoryError> {
        let next_h_count = self.h_count + d_cycle;
        trace!("setting h_blank to {}", next_h_count);

        if self.h_count < 240 && next_h_count >= 240 {
            self.h_count = next_h_count;
            debug!("Setting H_BLANK_FLAG hi");
            set_bit_high(ram, DISP_STAT, H_BLANK_FLAG);
            Ok(false)
        } else if self.h_count < 308 && next_h_count >= 308 {
            self.h_count = next_h_count - 308;
            debug!("Setting H_BLANK_FLAG low");
            set_bit_low(ram, DISP_STAT, H_BLANK_FLAG);
            Ok(true)
        } else {
            self.h_count = next_h_count;
            Ok(false)
        }
    }

    // if v goes from 227 to 0, the frame is done
    fn update_v_count(&mut self, ram: &mut SystemMemory) -> Result<bool, MemoryError> {
        self.v_count += 1;
        // TODO, well this propogate the error in set_bit_x?
        if self.v_count == 160 {
            debug!("Setting V_BLANK_FLAG hi");
            set_bit_high(ram, DISP_STAT, V_BLANK_FLAG);
        } else if self.v_count == 226 {
            debug!("Setting V_BLANK_FLAG low");
            set_bit_low(ram, DISP_STAT, V_BLANK_FLAG);
        } else if self.v_count == 228 {
            self.frame += 1;
            info!("Frame done {}", self.frame);
            self.v_count = 0;
        }

        debug!("Setting VCOUNT to {}", self.v_count);
        if self.v_count == 0 {
            ram.write_byte(V_COUNT, self.v_count).map(|_| true)
        } else {
            ram.write_byte(V_COUNT, self.v_count).map(|_| false)
        }
    }

    pub fn get_next_frame(&mut self, ram: &mut SystemMemory) -> Vec<u8> {
        let disp_control = display_control(ram).expect("Something went wrong grabbing the display control");
        let _: Vec<BgControl> = match get_bgs(&disp_control, ram) {
            Ok(b) => b,
            Err(e) => {
                panic!("Err occured: {}", e)
            }
        };

        //info!("Display Control: {:?}, BGs enabled: {}", disp_control, bgs.len());
        // Do stuff with BGs here:
        //

        // OBJ stuff
        let (objects, _)  = if disp_control.display_obj {
            get_objs_and_params(ram, &disp_control)
        } else {
            (Vec::new(), Vec::new())
        };

        // TODO: Objs start later in VRAM when bg_mode is bitmap modes
        let palettes = get_palettes(ram);

        for obj in objects {
            let palette = palettes.get_palette(obj.palette_idx);
            let tile_base = BASE_OAM + (obj.character_name * 32);

            // becuase each byte is 2 pixels
            //info!("Attemping to draw following OBJ: {:?}", obj);
            //info!("Using the following palette: {:?}", palette);
            for x in 0..(obj.obj_shape.h) { 
                for y in 0..(obj.obj_shape.w) {
                    // can probably progressively add stuff instead
                    let (r, g, b) = if obj.is_256_color {
                        let c_idx = get_color_id_256_colors_2d(x, y, tile_base, ram);
                        if c_idx == 0 {
                            continue;
                        }
                        palettes.get_256_color(c_idx)
                    }  else {
                        let c_idx = get_color_id_16_palette_2d(x, y, tile_base, ram);
                        palette[c_idx]
                    };

                    let buffer_idx = euclid_to_buffer_idx((x + obj.x_coord) as usize, (y + obj.y_coord) as usize);
                    if buffer_idx + 2 <= self.next_frame.len() {
                        self.next_frame[buffer_idx] = r;
                        self.next_frame[buffer_idx + 1] = g;
                        self.next_frame[buffer_idx + 2] = b;
                    }
                }
            }
        }

        self.next_frame.clone()
    }
} 

fn get_color_id_16_palette_2d(x: u32, y: u32, tile_base: u32, ram: &mut SystemMemory) -> usize {
    // Translating 
    let idx = tile_base + ((x % 8) >> 1) + ((x >> 3) * 0x40) + (0x4 * (y % 8)) + (0x400 * (y >> 3));
    let pixel_byte = ram.read_byte(idx as usize).expect("Error reading byte while writing to pixel buffer");
    if x & 1 == 0 {
        (pixel_byte & 0xf) as usize
    } else {
        ((pixel_byte >> 4) & 0xf) as usize
    }
}

fn get_color_id_256_colors_2d(x: u32, y: u32, tile_base: u32, ram: &mut SystemMemory) -> usize {
    let idx = tile_base + x % 8 + ((x >> 3) * 0x40) + (0x8 * (y % 8)) + (0x400 * (y >> 3));
    let pixel_byte = ram.read_byte(idx as usize).expect("Error reading byte while writing to pixel buffer");
    pixel_byte as usize
}

fn euclid_to_buffer_idx(x: usize, y: usize) -> usize {
    let bytes_in_row = WIDTH * 4;
    (x * 4) + (y * bytes_in_row)
}

fn get_bgs(disp_control: &DisplayControl, ram: &mut SystemMemory) -> Result<Vec<BgControl>, MemoryError> {
    let mut bgs: Vec<BgControl> = Vec::new();

    match disp_control.bg_mode {
        0 => {
            if disp_control.display_bg0 { bgs.push(bg_control0(ram)?); }
            if disp_control.display_bg1 { bgs.push(bg_control1(ram)?); }
            if disp_control.display_bg2 { bgs.push(bg_control2(ram)?); }
            if disp_control.display_bg3 { bgs.push(bg_control3(ram)?); }
        },
        1 => {
            if disp_control.display_bg0 { bgs.push(bg_control0(ram)?); }
            if disp_control.display_bg1 { bgs.push(bg_control1(ram)?); }
            if disp_control.display_bg2 { bgs.push(bg_control2(ram)?); }
        },
        2 => {
            if disp_control.display_bg2 { bgs.push(bg_control2(ram)?); }
            if disp_control.display_bg3 { bgs.push(bg_control3(ram)?); }
        }
        3 | 4| 5 => {
            if disp_control.display_bg2 { bgs.push(bg_control2(ram)?); }
        }
        _ => {
            error!("Background Mode cannot be more than 5");
            panic!()
        }
    }
    Ok(bgs)
}

// Where do we start reading, and how many do we read?
// We can't really return a buffer since it can be behind a background
fn get_objs_and_params(ram: &mut SystemMemory, display_control: &DisplayControl) -> (Vec<OamAttribute>, Vec<RotationScaleParameter>) {
    let oam = ram.get_oam();
    let mut objs: Vec<OamAttribute> = Vec::new();
    let mut param_builder = RotationScaleParameterBuilder::new();

    for chunk in oam.chunks(2) {
        if chunk[0] != 0 && (chunk[1] & 0xffff) != 0 {
            objs.push(OamAttribute::from(chunk));
        }
        param_builder.add_parameter(chunk[1]);
    }
    let params = param_builder.build();

    (objs, params)
}

struct Mosaic {
    bg_h: i32,
    bg_v: i32,
    obj_h: i32,
    obj_v: i32,
}

impl From<u32> for Mosaic {
    fn from(value: u32) -> Self {
        Mosaic {
            bg_h: (value.half_byte_at(0) as i32) - 1,
            bg_v: (value.half_byte_at(4) as i32) - 1,
            obj_h: (value.half_byte_at(8) as i32) - 1,
            obj_v: (value.half_byte_at(12) as i32) - 1,
        }
    }
}
