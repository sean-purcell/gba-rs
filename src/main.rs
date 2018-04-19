extern crate byteorder;
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate memmap;
extern crate sdl2;

extern crate flame;

use std::fs::File;
use std::ffi::OsStr;
use std::path::Path;

use clap::{App, Arg};

mod bit_util;
mod shared;

mod cpu;
mod io;
mod mmu;
mod rom;

mod gba;

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
        .arg(Arg::with_name("rom").required(true).help(
            "ROM file to emulate",
        ))
        .arg(Arg::with_name("profile")
            .short("p")
            .long("profile")
            .required(false)
            .takes_value(true)
            .value_name("mode")
            .possible_values(&["file", "html"]))
        .get_matches();

    let res = run_gba(app_m.value_of_os("rom").unwrap());

    match app_m.value_of("profile") {
        Some("html") => flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap(),
        Some("file") => flame::dump_text_to_writer(&mut File::create("flame-graph.txt").unwrap()).unwrap(),
        _ => (),
    };

    res
}

fn run_gba(path: &OsStr) -> Result<()> {
    let path = Path::new(path);

    let rom = rom::GameRom::new(&path)?;

    let mut gba = gba::Gba::new(rom);

    /*
    gba.run();
    let mut event_pump = gba.ctx.event_pump().unwrap();
    let mut i = 0;
    loop {
        use sdl2;
        i = (i + 1) % 255;
        gba.canvas.set_draw_color(
            sdl2::pixels::Color::RGB(i, 64, 255 - i),
        );
        gba.canvas.clear();
        gba.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        gba.canvas.draw_point(sdl2::rect::Point::new(120, 80));
        event_pump.pump_events();
        let keys = event_pump.keyboard_state();
        if keys.is_scancode_pressed(sdl2::keyboard::Scancode::Escape) {
            break;
        }
        gba.canvas.present();

        std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
    */

    gba.run()
}

#[cfg(test)]
mod test;
