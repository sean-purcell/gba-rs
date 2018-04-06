extern crate byteorder;
extern crate memmap;

use std::path::Path;

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

    let rom = rom::Rom::new(&path);

    println!("ROM: {:?}", &rom);

    let mmu = mmu::Mmu::new();
    let cpu = cpu::Cpu::new(mmu);

    Ok(())
}

fn main() {
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
