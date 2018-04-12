extern crate byteorder;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate maplit;
extern crate memmap;

use std::boxed::Box;
use std::path::Path;

mod bit_util;
mod cpu;
mod mmu;
mod rom;

#[derive(Debug)]
pub enum GBAError {
    RomLoadError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, GBAError>;

fn run_gba() -> Result<()> {
    let path = Path::new("roms/minish_cap.gba");

    let rom = rom::GameRom::new(&path);

    println!("ROM: {:?}", &rom);

    const MEM_SIZE: usize = 0x10000;

    let mmu = mmu::ram::Ram::new(MEM_SIZE);
    let cpu = cpu::Cpu::new(Box::new(mmu), &vec![(0u8, 0u32)]);

    Ok(())
}

fn main() {
    env_logger::init();

    use GBAError::*;
    match run_gba() {
        Ok(_) => {}
        Err(errcode) => {
            match errcode {
                RomLoadError(err) => println!("ROM failed to load: {:?}", err),
            }
        }
    }
}
