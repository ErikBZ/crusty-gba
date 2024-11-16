mod gba;
mod utils;
mod cli;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

extern crate strum_macros;

use clap::Parser;
use cli::Args;
use gba::Conditional;
use gba::cpu::CPU;
use gba::debugger::{DebuggerCommand, ContinueSubcommand};
use gba::system::SystemMemory;
use gba::arm::decode_as_arm;
use gba::thumb::decode_as_thumb;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    event_loop::EventLoop,
    dpi::LogicalSize,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

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
    let cpu = CPU::default();
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

    debug_bios(cpu, memory);
    Ok(())
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
                    println!("{:x}", cpu.pc());
                    cpu.tick(&mut memory);
                    i += 1;
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
