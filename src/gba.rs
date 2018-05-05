use std::boxed::Box;
use std::default::Default;
use std::mem;
use std::ptr;
use std::time::{Duration, Instant};
use std::thread;

use flame;

use sdl2;
use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use shared::Shared;

use Result;

use cpu::Cpu;
use cpu::mode::Mode;
use io::IoReg;
use io::key::KeyState;
use io::ppu::{Ppu, ROWS, COLS};
use mmu::gba::Gba as GbaMmu;
use rom::GameRom;

const CYCLES_PER_SEC: u64 = 16 * 1024 * 1024;
const CYCLES_PER_FRAME: u64 = 280896;

#[derive(Clone, Debug)]
pub struct Options {
    pub fps_limit: bool,
    pub breaks: Vec<u32>,
    pub step_frames: bool,
    pub direct_boot: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            fps_limit: true,
            breaks: Default::default(),
            step_frames: false,
            direct_boot: false,
        }
    }
}

/// Parent container for all components of the system
pub struct Gba<'a> {
    opts: Options,

    pub ctx: Sdl,

    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    texture: Texture<'a>,

    cpu: Cpu<GbaMmu<'a>>,
    mmu: GbaMmu<'a>,
    io: IoReg<'a>,
    ppu: Ppu<'a>,
}

impl<'a> Gba<'a> {
    pub fn new(rom: GameRom, bios: GameRom, options: Options) -> Box<Self> {
        unsafe {
            let mut gba: Box<Gba> = Box::new(mem::uninitialized());
            ptr::write(&mut gba.opts, options);

            ptr::write(&mut gba.ctx, sdl2::init().unwrap());
            let video = gba.ctx.video().unwrap();
            let window = video
                .window("GBA", 720, 480)
                .position_centered()
                .build()
                .unwrap();

            ptr::write(&mut gba.canvas, window.into_canvas().build().unwrap());
            gba.canvas.set_logical_size(COLS, ROWS).unwrap();
            ptr::write(&mut gba.texture_creator, gba.canvas.texture_creator());
            info!(
                "Default pixel format: {:?}",
                gba.texture_creator.default_pixel_format()
            );
            ptr::write(
                &mut gba.texture,
                mem::transmute(
                    gba.texture_creator
                        .create_texture_streaming(PixelFormatEnum::RGB888, COLS, ROWS)
                        .unwrap(),
                ),
            );

            ptr::write(&mut gba.io, IoReg::new());
            ptr::write(
                &mut gba.mmu,
                GbaMmu::new(rom, bios, Shared::new(&mut gba.io)),
            );

            use cpu::reg;
            let mmu = Shared::new(&mut gba.mmu);
            ptr::write(&mut gba.cpu, Cpu::new(mmu, &[]));
            if gba.opts.direct_boot {
                gba.cpu.init_direct();
            } else {
                gba.cpu.init_arm();
            }
            let opts = Shared::new(&mut gba.opts);
            gba.cpu.set_breaks(opts.breaks.iter());

            ptr::write(
                &mut gba.ppu,
                Ppu::new(
                    Shared::new(&mut gba.texture),
                    Shared::new(&mut gba.io),
                    Shared::new(&mut gba.mmu),
                ),
            );

            let cpu = Shared::new(&mut gba.cpu);
            let ppu = Shared::new(&mut gba.ppu);
            gba.io.init(cpu, mmu, ppu);


            gba
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut frame = 0;
        let mut event_pump = self.ctx.event_pump().unwrap();

        let frame_duration = Duration::new(
            0,
            ((1_000_000_000u64 * CYCLES_PER_FRAME) / CYCLES_PER_SEC) as u32,
        );
        let mut prev_time = Instant::now();
        loop {
            let _guard = flame::start_guard("frame cycle");
            let start = Instant::now();

            flame::span_of("frame emu", || self.emulate_frame());
            flame::span_of("frame copy", || {
                self.canvas.copy(&self.texture, None, None).unwrap()
            });
            flame::span_of("frame present", || self.canvas.present());

            {
                event_pump.pump_events();
                let keys = event_pump.keyboard_state();
                self.io.set_keyreg(&KeyState::new_from_keystate(&keys));

                if keys.is_scancode_pressed(sdl2::keyboard::Scancode::Escape) {
                    break;
                }
                if keys.is_scancode_pressed(sdl2::keyboard::Scancode::B) {
                    use log;
                    log::set_max_level(match log::max_level() {
                        log::LevelFilter::Debug => log::LevelFilter::Error,
                        _ => log::LevelFilter::Debug,
                    });
                }
            }
            if self.opts.step_frames {
                info!("Frame: {}", frame);
                loop {
                    let event = event_pump.wait_event();
                    if let sdl2::event::Event::KeyDown { scancode, .. } = event {
                        if scancode == Some(sdl2::keyboard::Scancode::F) {
                            break;
                        }
                    }
                }
            }

            let end = Instant::now();
            if self.opts.fps_limit {
                if end < prev_time + frame_duration {
                    let sleep_time = (prev_time + frame_duration) - end;
                    thread::sleep(sleep_time);
                }
            }
            prev_time = prev_time + frame_duration;

            let now = Instant::now();
            info!("{} fps", 1_000_000_000u32 / ((now - start).subsec_nanos()));
            frame += 1;
        }
        Ok(())
    }

    fn emulate_frame(&mut self) {
        for _ in 0..CYCLES_PER_FRAME {
            self.cycle();
        }
    }

    fn cycle(&mut self) {
        self.cpu.cycle();
        self.ppu.cycle();
        self.io.cycle();
    }
}
