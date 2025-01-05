mod gba;
mod utils;
mod cli;
mod ppu;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

extern crate strum_macros;

use clap::Parser;
use cli::Args;
use gba::cpu::CPU;
use crate::ppu::PPU;
use gba::debugger::{DebuggerCommand, ContinueSubcommand};
use gba::system::SystemMemory;
use std::time::Instant;
use tracing::{error, event, info, Level};
use tracing_subscriber::{filter, fmt, reload, reload::Handle, prelude::*, Registry};
use tracing_subscriber::filter::LevelFilter;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    dpi::LogicalSize,
    window::WindowBuilder,
    keyboard::KeyCode,
};
use winit_input_helper::WinitInputHelper;
const WIDTH: u32 = 240;
const HEIGHT: u32 = 160;
const CYCLES_PER_SCANLINE: u32 = 4 * 240;
const CYCLES_PER_FRAME: u32 = 4 * 240 * 226;

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
        cli::Renderer::Terminal => debug_bios(cpu, memory, ppu, reload_handle),
        cli::Renderer::Gui => {
            let _ = run_gui(cpu, memory, reload_handle);
            ()
        },
    };

    Ok(())
}

fn run_gui(mut cpu: CPU, mut memory: SystemMemory, reload_handle: Handle<LevelFilter, Registry>)  -> Result<(), Box<dyn std::error::Error> >{
    event!(Level::INFO, "Runing GUI");
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let mut ppu = PPU::default();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("Crusty Gameboy")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let _res = event_loop.run(|event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let current = Instant::now();
                loop {
                    cpu.tick(&mut memory);
                    if ppu.tick(cpu.cycles(), &mut memory) {
                        break;
                    }

                }
                {
                    let ppu_buffer = ppu.get_next_frame(&mut memory);
                    let frame = pixels.frame_mut();
                    let mut i = 0;
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel[0] = ppu_buffer[i];
                        pixel[1] = ppu_buffer[i + 1];
                        pixel[2] = ppu_buffer[i + 2];
                        pixel[3] = ppu_buffer[i + 3];
                        i += 4;
                    }
                }
                let _ = pixels.render();
                // TODO: This seems wrong?
                let dt = Instant::now() - current;
                if dt.as_secs_f64() > 0.0 {
                    std::thread::sleep(dt);
                }

            },
            _ => (),
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }
            if input.key_pressed(KeyCode::Space) {
                let _ = reload_handle.modify(|filter| *filter = filter::LevelFilter::DEBUG);
            }
            window.request_redraw();
        }
    });
    Ok(())
}

fn debug_bios(mut cpu: CPU, mut memory: SystemMemory, mut ppu: PPU, reload_handle: Handle<LevelFilter, Registry>) {
    event!(Level::INFO, "Runing Debug session");
    use std::io;
    let mut break_points: HashSet<usize> = HashSet::new();

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {},
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }

        let cmd = match DebuggerCommand::parse(&input) {
            Ok(dc) => dc,
            Err(e) => {
                println!("{}", e);
                continue;
            },
        };

        match cmd {
            DebuggerCommand::BreakPoint(address) => {
                if break_points.contains(&address) {
                    break_points.remove(&address);
                } else {
                    break_points.insert(address);
                    println!("{:?}", break_points);
                }
            },
            DebuggerCommand::Continue(ContinueSubcommand::Endless) => {
                cpu.tick(&mut memory);
                ppu.tick(cpu.cycles(), &mut memory);

                while !break_points.contains(&cpu.instruction_address()) {
                    cpu.tick(&mut memory);
                    if ppu.tick(cpu.cycles(), &mut memory) {
                        println!("{}", cpu);
                    }
                }
                println!("{}", cpu);
            },
            DebuggerCommand::Continue(ContinueSubcommand::For(l)) => {
                let mut n = 0;
                while !break_points.contains(&cpu.instruction_address()) && l > n {
                    cpu.tick(&mut memory);
                    ppu.tick(cpu.cycles(), &mut memory);

                    println!("{}", cpu);
                    n += 1;
                }
            },
            DebuggerCommand::Next => {
                cpu.tick(&mut memory);
                ppu.tick(cpu.cycles(), &mut memory);

                println!("{}", cpu);
            },
            DebuggerCommand::Info => {
                println!("{}", cpu);
            },
            DebuggerCommand::Quit => break,
            DebuggerCommand::LogLevel(lf) => {
                let _ = reload_handle.modify(|filter| *filter = lf);
            },
            DebuggerCommand::ReadMem(address) => {
                match memory.read_word(address) {
                    Ok(d) =>  println!("{:x}: {:x}", address, d),
                    Err(e) => println!("{}", e),
                }
            }
            _ => (),
        }
    }
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

