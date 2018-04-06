extern crate memmap;

use std::fs::File;
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

    /*
    let f = File::open(&path).unwrap();

    let rom = unsafe { memmap::Mmap::map(&f).unwrap() };

    println!("ROM size: {}", rom.len());
    println!("Fixed value: {:x}", rom[0xb2]);
    */

    let rom = rom::Rom::new(&path);

    println!("ROM: {:?}", &rom);

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
