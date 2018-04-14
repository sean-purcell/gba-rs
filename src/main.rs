extern crate byteorder;
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate memmap;

use std::boxed::Box;
use std::ffi::OsStr;
use std::path::Path;

use clap::{App,Arg};

mod bit_util;
mod cpu;
mod mmu;
mod rom;

fn main() {
    env_logger::init();

    use GBAError::*;
    match run_emu() {
        Ok(_) => {}
        Err(errcode) => {
            match errcode {
                RomLoadError(err) => println!("ROM failed to load: {:?}", err),
            }
        }
    }
}

#[derive(Debug)]
pub enum GBAError {
    RomLoadError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, GBAError>;

fn run_emu() -> Result<()> {
    let app_m = App::new("gba-rs")
        .version("0.1")
        .about("Bad GBA Emulator")
        .author("Sean Purcell")
        .arg(Arg::with_name("rom")
             .required(true)
             .help("ROM file to emulate"))
        .get_matches();

    run_game(app_m.value_of_os("rom").unwrap())
}

fn run_game(path: &OsStr) -> Result<()> {
    let path = Path::new(path);

    let rom = rom::GameRom::new(&path)?;

    println!("ROM: {:?}", &rom);

    let mmu = mmu::gba::Gba::new_with_rom(rom);

    use cpu::reg;
    let mut cpu = cpu::Cpu::new(Box::new(mmu),
        &[(reg::PC, 0x08000000),
          (reg::SP, 0x03007F00)]);

    cpu.run();

    Ok(())
}

#[cfg(test)]
mod test;
