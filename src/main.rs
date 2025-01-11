mod gba;
mod utils;
mod cli;
mod ppu;
mod renderer;

use std::fs::File;
use std::io::prelude::*;
use clap::Parser;
use cli::Args;
use gba::cpu::CPU;
use crate::ppu::PPU;
use gba::system::SystemMemory;
use tracing::{error, event, Level};
use tracing_subscriber::{fmt, reload, prelude::*};
use tracing_subscriber::filter::LevelFilter;
use crate::renderer::{run_gui, run_debug, run_ratatui};

use pixels::Error;

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let filter: LevelFilter = match args.log_level {
        Some(c) => c.into(),
        None => LevelFilter::OFF,
    };

    let (filter, reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry().with(filter).with(fmt::Layer::default()).init();

    // TODO: Just put test.gba in the root dir
    let mut bios_rom = match File::open(args.bios) {
        Ok(f) => f,
        Err(e) => {
            error!("Unable to to open bios file: {:?}", e);
            return Ok(());
        }
    };

    let mut game_rom = match File::open(args.game) {
        Ok(f) => f,
        Err(e) => {
            error!("Unable to to open gba file: {:?}", e);
            return Ok(());
        }
    };

    let bios: Vec<u32> = read_file_into_u32(&mut bios_rom);
    let game_pak: Vec<u32> = read_file_into_u32(&mut game_rom);
    let cpu = CPU::default();
    let ppu = PPU::default();
    let mut memory = SystemMemory::new();
    memory.copy_bios(bios);
    memory.copy_game_pak(game_pak);
    event!(Level::INFO, "Copied the stuff over");

    match args.render {
        cli::Renderer::Debug => run_debug(cpu, memory, ppu, reload_handle),
        cli::Renderer::Gui => {
            let _ = run_gui(cpu, memory, reload_handle);
            ()
        },
        cli::Renderer::Ratatui => {
            let _ = run_ratatui();
            ()
        },
    };

    Ok(())
}

fn read_file_into_u32(file: &mut File) -> Vec<u32> {
    let mut instructions = Vec::new();

    loop {
        let mut buffer = [0; 4];

        let n = match file.take(4).read(&mut buffer) {
            Ok(n) => n,
            Err(_) => break,
        };

        if n == 0 {
            break;
        }
        instructions.push(u32::from_le_bytes(buffer));
        if n < 4 {
            break;
        }
    }

    instructions
}

