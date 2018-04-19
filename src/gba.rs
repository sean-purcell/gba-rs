use std::boxed::Box;
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
use io::IoReg;
use io::ppu::Ppu;
use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;
use rom::GameRom;

const CYCLES_PER_SEC: u64 = 16 * 1024 * 1024;
const CYCLES_PER_FRAME: u64 = 280896;

/// Parent container for all components of the system
pub struct Gba<'a> {
    pub ctx: Sdl,

    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    texture: Texture<'a>,

    cpu: Cpu<GbaMmu>,
    mmu: GbaMmu,
    io: IoReg,
    ppu: Ppu<'a>,
}

impl<'a> Gba<'a> {
    pub fn new(rom: GameRom) -> Box<Gba<'a>> {
        unsafe {
            let mut gba: Box<Gba> = Box::new(mem::uninitialized());

            ptr::write(&mut gba.ctx, sdl2::init().unwrap());
            let video = gba.ctx.video().unwrap();
            let window = video
                .window("GBA", 720, 480)
                .position_centered()
                .build()
                .unwrap();

            ptr::write(&mut gba.canvas, window.into_canvas().build().unwrap());
            gba.canvas.set_logical_size(240, 160).unwrap();
            ptr::write(&mut gba.texture_creator, gba.canvas.texture_creator());
            info!(
                "Default pixel format: {:?}",
                gba.texture_creator.default_pixel_format()
            );
            ptr::write(
                &mut gba.texture,
                mem::transmute(
                    gba.texture_creator
                        .create_texture_streaming(PixelFormatEnum::RGB888, 240, 160)
                        .unwrap(),
                ),
            );

            ptr::write(&mut gba.io, IoReg::new());
            ptr::write(&mut gba.mmu, GbaMmu::new(rom, Shared::new(&mut gba.io)));

            use cpu::reg;
            ptr::write(
                &mut gba.cpu,
                Cpu::new(
                    Shared::new(&mut gba.mmu),
                    &[(reg::PC, 0x08000000), (reg::SP, 0x03007F00)],
                ),
            );

            ptr::write(
                &mut gba.ppu,
                Ppu::new(
                    Shared::new(&mut gba.texture),
                    Shared::new(&mut gba.io),
                    Shared::new(&mut gba.mmu.vram),
                ),
            );

            gba
        }
    }

    pub fn run(&mut self) -> Result<()> {
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

            event_pump.pump_events();
            let keys = event_pump.keyboard_state();
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::Escape) {
                break;
            }

            let now = Instant::now();
            info!("{} fps", 1_000_000_000u32 / ((now - start).subsec_nanos()));
            let end = Instant::now();
            if end < prev_time + frame_duration {
                let sleep_time = (prev_time + frame_duration) - end;
                //thread::sleep(sleep_time);
            }
            prev_time = prev_time + frame_duration;
        }
        Ok(())
    }

    fn emulate_frame(&mut self) {
        for i in 0..CYCLES_PER_FRAME {
            self.cycle();
        }
    }

    fn cycle(&mut self) {
        self.cpu.cycle();
        self.ppu.cycle();
    }
}
