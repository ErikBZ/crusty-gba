mod gba;
use std::fs::File;
use std::io::prelude::*;

extern crate strum_macros;

use gba::instructions::Opcode;

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
    println!("Size of file: {:?}", codes.len() * 4);

    for i in 0..100 {
        let op = Opcode::from(codes[i]);
        if let Opcode::Undef(inst) = op {
            println!("{:0x}", inst);
        } else {
            println!("{:0b} {:0x} {}", codes[i], codes[i], op.string_repr());
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
