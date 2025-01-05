mod disp_control;
mod bg_control;
mod window_control;
mod color_effect;

use bg_control::{bg_control0, bg_control1, bg_control2, bg_control3, BgControl};
use tracing::{warn, trace, info, debug, error};
use crate::{gba::system::MemoryError, utils::Bitable, SystemMemory};
use disp_control::{display_control, DisplayControl};
// Base off of https://github.com/tuzz/game-loop 

// Values for controlling how the PPU draws pixels to the display
const DISP_CONTROL: usize = 0x4000000;
const DISP_STAT_ADDR: usize = 0x4000004;
const V_COUNT_ADDR: usize = 0x4000006;

const V_BLANK_FLAG: u32 = 0b00000001;
const H_BLANK_FLAG: u32 = 0b00000010;
const V_COUNTER_FLAG: u32 = 0b00000100;

fn set_bit_high(ram: &mut SystemMemory, addr: usize, flag: u32) -> Result<(), MemoryError> {
    let data = ram.read_halfword(addr)?;
    ram.write_halfword(addr, data | flag)
}

fn set_bit_low(ram: &mut SystemMemory, addr: usize, flag: u32) -> Result<(), MemoryError> {
    let data = ram.read_halfword(addr)?;
    ram.write_halfword(addr, data & !flag)
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
            next_frame: vec![20; 240 * 160 * 4]
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

        if self.h_count < 960 && next_h_count >= 960 {
            self.h_count = next_h_count;
            debug!("Setting H_BLANK_FLAG hi");
            set_bit_high(ram, DISP_STAT_ADDR, H_BLANK_FLAG).map(|_| false)
        } else if self.h_count < 1232 && next_h_count >= 1232 {
            self.h_count = next_h_count - 1232;
            debug!("Setting H_BLANK_FLAG low");
            set_bit_low(ram, DISP_STAT_ADDR, H_BLANK_FLAG).map(|_| true)
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
            set_bit_high(ram, DISP_STAT_ADDR, V_BLANK_FLAG)?;
        } else if self.v_count == 226 {
            debug!("Setting V_BLANK_FLAG low");
            set_bit_low(ram, DISP_STAT_ADDR, V_BLANK_FLAG)?;
        } else if self.v_count == 228 {
            self.frame += 1;
            info!("Frame done {}", self.frame);
            self.v_count = 0;
        }

        debug!("Setting VCOUNT to {}", self.v_count);
        if self.v_count == 0 {
            ram.write_byte(V_COUNT_ADDR, self.v_count).map(|_| true)
        } else {
            ram.write_byte(V_COUNT_ADDR, self.v_count).map(|_| false)
        }
    }

    pub fn get_next_frame(&mut self, ram: &mut SystemMemory) -> Vec<u8> {
        let disp_control = display_control(ram).expect("Something went wrong grabbing the display control");
        let bgs: Vec<BgControl> = match get_bgs(&disp_control, ram) {
            Ok(b) => b,
            Err(e) => {
                panic!("Err occured: {}", e)
            }
        };

        self.next_frame.clone()
    }
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
