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
use gba::Conditional;
use gba::cpu::CPU;
use gba::debugger::{DebuggerCommand, ContinueSubcommand};
use gba::system::{MemoryError, SystemMemory};
use gba::arm::decode_as_arm;
use gba::thumb::decode_as_thumb;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    dpi::LogicalSize,
    window::WindowBuilder,
    keyboard::KeyCode,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;
const CYCLES_PER_SCANLINE: u32 = 4 * 240;
const CYCLES_PER_FRAME: u32 = 4 * 240 * 226;

fn main() -> Result<(), Error> {
    let args = Args::parse();

    // TODO: Just put test.gba in the root dir
    let mut bios_rom = match File::open(args.bios) {
        Ok(f) => f,
        Err(e) => {
            println!("Unable to to open bios file: {:?}", e);
            return Ok(());
        }
    };

    let mut game_rom = match File::open(args.game) {
        Ok(f) => f,
        Err(e) => {
            println!("Unable to to open gba file: {:?}", e);
            return Ok(());
        }
    };

    let bios: Vec<u32> = read_file_into_u32(&mut bios_rom);
    let game_pak: Vec<u32> = read_file_into_u32(&mut game_rom);
    let mut cpu = CPU::default();
    let mut memory = SystemMemory::default();
    memory.copy_bios(bios);
    memory.copy_game_pak(game_pak);

    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();

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
    let mut counter = 0;

    let _res = event_loop.run(|event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                next_frame(&mut cpu, &mut memory);
                counter += 100;
                {
                    let buffer = pixels.frame_mut();
                    for i in (counter - 100)..counter {
                        let idx = i % buffer.len();
                        buffer[idx] = 0x88;
                    }
                }

                let _ = pixels.render();
            },
            _ => (),
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }
            window.request_redraw();
        }
    });

    debug_bios(cpu, memory);
    Ok(())
}

fn next_frame(cpu: &mut CPU, ram: &mut SystemMemory ) {
    for i in 0..227 {
        cpu.tick_for_cycles(ram, CYCLES_PER_SCANLINE);
        let _ = ram.write_halfword(0x4000006, i);
    }
}

fn debug_bios(mut cpu: CPU, mut memory: SystemMemory) {
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
                let mut i = 0;
                while !break_points.contains(&cpu.pc()) {
                    cpu.tick(&mut memory);
                    i += 1;

                    if cpu.pc() == 0x1776 || cpu.pc() == 0x1778{
                        panic!();
                    }
                }
            },
            DebuggerCommand::Continue(ContinueSubcommand::For(l)) => {
                let mut n = 0;
                while !break_points.contains(&cpu.pc()) && l > n {
//                    println!("{:x}", cpu.pc());
                    cpu.tick(&mut memory);
                    n += 1;
                }
            },
            DebuggerCommand::Next => {
                cpu.tick(&mut memory);
                let op = if !cpu.is_thumb_mode() {
                    decode_as_arm(cpu.decode)
                } else {
                    decode_as_thumb(cpu.decode)
                };
                let cond = Conditional::from(cpu.decode);

                println!("{}", cpu);
                if cpu.is_thumb_mode() {
                    println!("{:#04x} {:?} {:?}", cpu.decode, cond, op);
                } else {
                    println!("{:#08x} {:?} {:?}", cpu.decode, cond, op);
                }
            },
            DebuggerCommand::Info => {
                let op = if !cpu.is_thumb_mode() {
                    decode_as_arm(cpu.decode)
                } else {
                    decode_as_thumb(cpu.decode)
                };
                let cond = Conditional::from(cpu.decode);

                println!("{}", cpu);
                if cpu.is_thumb_mode() {
                    println!("{:#04x} {:?} {:?}", cpu.decode, cond, op);
                } else {
                    println!("{:#08x} {:?} {:?}", cpu.decode, cond, op);
                }            },
            DebuggerCommand::Quit => break,
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

