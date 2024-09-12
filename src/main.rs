mod gba;
use std::fs::File;
use std::io::prelude::*;

extern crate strum_macros;

use gba::arm::ArmInstruction;
use gba::thumb::ThumbInstruction;
use gba::cpu::CPU;
use gba::debugger::DebuggerCommand;

fn main() {
    // TODO: Just put test.gba in the root dir
    let mut file = match File::open("test.gba") {
        Ok(f) => f,
        Err(e) => {
            println!("There was an error: {:?}", e);
            return;
        }
    };
    let codes: Vec<u32> = read_file_into_u32(&mut file);
    debug_bios(codes);
}

fn debug_bios(codes: Vec<u32>) {
    use std::io;
    let mut cpu = CPU::default();
    let mut ram: [u32;128] = [0;128];

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
            DebuggerCommand::BreakPoint(_) => println!("Adding break point"),
            DebuggerCommand::Continue => println!("Continuing"),
            DebuggerCommand::Next => {
                let i = (cpu.pc() >> 2) as usize;
                if i < codes.len() {
                    let inst = codes[(cpu.pc() >> 2) as usize];
                    cpu.run_instruction(inst, &mut ram);
                } else {
                    println!("Address it not within ROM");
                    continue;
                }
            },
            DebuggerCommand::Info => {
                let i = (cpu.pc() >> 2) as usize;
                let op = if i < codes.len() {
                    ArmInstruction::from(codes[(cpu.pc() >> 2) as usize])
                } else {
                    println!("Address it not within ROM");
                    continue;
                };
                println!("{}", cpu);
                println!("{:?}", op);
            },
            DebuggerCommand::Quit => break,
        }
    }
}

fn dump_opcodes(num_of_lines: usize, codes: Vec<u32>) -> Result<(), ()> {
    if num_of_lines < codes.len() {
        return Err(())
    };
    let mut decode_thumb = false;

    for i in 0..0x100 {
        let inst_address = i << 2;
        if !decode_thumb {
            let op = ArmInstruction::from(codes[i]);
            match &op {
                ArmInstruction::BX(_, _) => {
                    println!("{:#08x} {:0x} {}", inst_address, codes[i], op.string_repr());
                    decode_thumb = true;
                }
                ArmInstruction::Undef(_) => {},
                _ => println!("{:#08x} {:0x} {}", inst_address, codes[i], op.string_repr()),
            }
        } else {
            let op2 = ThumbInstruction::from((codes[i] >> 16) as u16);
            let op1 = ThumbInstruction::from((codes[i] & 0xffff) as u16);
            match op1 {
                ThumbInstruction::Undefined => decode_thumb = false,
                _ => println!("{:#08x} {:0x} {:?}", inst_address, codes[i], op1),
            };
            match op2 {
                ThumbInstruction::Undefined => decode_thumb = false,
                _ => println!("{:#08x} {:0x} {:?}", inst_address+2, codes[i], op2),
            }
        }
    }
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
