extern crate bincode;
extern crate byteorder;
extern crate clap;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate memmap;
extern crate sdl2;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate zstd;

extern crate flame;

use std::default::Default;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use clap::{App, Arg, ArgMatches};

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
        .arg(Arg::with_name("bios").required(true).help(
            "GBA bios rom to use",
        ))
        .arg(Arg::with_name("rom").required(true).help(
            "ROM file to emulate",
        ))
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .required(false)
                .takes_value(true)
                .value_name("mode")
                .possible_values(&["file", "html"])
                .help("Whether to write out flame values to file and how"),
        )
        .arg(
            Arg::with_name("fps-limit")
                .short("f")
                .long("fps-limit")
                .required(false)
                .takes_value(true)
                .value_name("bool")
                .possible_values(&["true", "false"])
                .default_value("true")
                .help(
                    "If true, limits the frame-rate to the GBA frame rate (~60fps)",
                ),
        )
        .arg(
            Arg::with_name("breakpoints")
                .short("b")
                .long("breaks")
                .required(false)
                .takes_value(true)
                .multiple(true)
                .use_delimiter(true)
                .validator(|s| match u32::from_str_radix(s.as_str(), 16) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err.description().to_string()),
                })
                .help("A list of addresses to warn when the CPU hits"),
        )
        .arg(Arg::with_name("step-frames").short("p").long("step").help(
            "Step through the frames step by step with the F key",
        ))
        .arg(Arg::with_name("quiet").short("q").long("quiet").multiple(true).help(
            "Reduces logging level by one from env settings (multiple allowed)",
        ))
        .arg(Arg::with_name("direct").short("d").long("direct").help(
            "Boot directly to the ROM instead of booting the BIOS",
        ))
        .arg(Arg::with_name("save-file")
             .short("s")
             .long("save")
             .required(false)
             .takes_value(true)
             .default_value("save")
             .help("The save file prefix to save to"),
        )
        .arg(Arg::with_name("save-type")
             .short("t")
             .long("type")
             .required(false)
             .takes_value(true)
             .possible_values(&["bin", "json"])
             .default_value("bin")
             .help("The type of save file to create")
        )
        .get_matches();

    for _ in 0..app_m.occurrences_of("quiet") {
        info!("Reduce logging");
        reduce_logging();
    }

    let res = run_gba(&app_m);

    match app_m.value_of("profile") {
        Some("html") => flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap(),
        Some("file") => {
            flame::dump_text_to_writer(&mut File::create("flame-graph.txt").unwrap()).unwrap()
        }
        _ => (),
    };

    res
}

fn run_gba(app_m: &ArgMatches) -> Result<()> {
    let bios_path = Path::new(app_m.value_of_os("bios").unwrap());
    let game_path = Path::new(app_m.value_of_os("rom").unwrap());

    let bios = rom::GameRom::new(&bios_path)?;
    let rom = rom::GameRom::new(&game_path)?;

    let breaks: Vec<u32> = match app_m.values_of("breakpoints") {
        Some(v) => v.map(|s| u32::from_str_radix(s, 16).unwrap()).collect(),
        None => vec![],
    };

    let opts = gba::Options {
        fps_limit: app_m.value_of("fps-limit").unwrap() == "true",
        breaks: breaks,
        step_frames: app_m.is_present("step-frames"),
        direct_boot: app_m.is_present("direct"),
        save_file: app_m.value_of_os("save-file").unwrap().to_os_string(),
        json_save: app_m.value_of("save-type").unwrap() == "json",
        ..Default::default()
    };

    let mut gba = gba::Gba::new(rom, bios, opts);

    gba.run()
}

fn reduce_logging() {
    use log::LevelFilter::*;
    log::set_max_level(match log::max_level() {
        Off | Error => Off,
        Warn => Error,
        Info => Warn,
        Debug => Info,
        Trace => Debug,
    });
}

#[cfg(test)]
mod test;
