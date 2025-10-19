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

    let mut cpu = CPU::default();
    let mut memory = SystemMemory::new();
    if let Some(bios_rom) = args.bios {
        let mut bios_rom_f = File::open(bios_rom).expect("Unable to open bios file");
        memory.copy_bios(read_file_into_u32(&mut bios_rom_f));
        cpu.reset_cpu_with_bios();
    } else {
        cpu.reset_cpu();
    }

    let mut game_rom = File::open(args.game).expect("Unable to open GBA file");
    memory.copy_game_pak(read_file_into_u32(&mut game_rom));

    let ppu = PPU::default();
    event!(Level::INFO, "Copied the stuff over");

    match args.render {
        cli::Renderer::Debug => run_debug(cpu, memory, ppu, reload_handle),
        cli::Renderer::Gui => {
            let _ = run_gui(cpu, memory, reload_handle);
        },
        cli::Renderer::Ratatui => {
            let _ = run_ratatui();
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

