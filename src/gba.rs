use std::boxed::Box;
use std::mem;
use std::ptr;
use std::time::Duration;
use std::thread;

use sdl2;
use sdl2::Sdl;
use sdl2::render::Canvas;
use sdl2::video::Window;

use shared::Shared;

use Result;

use cpu::Cpu;
use io::ppu::Ppu;
use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;
use rom::GameRom;

/// Parent container for all components of the system
pub struct Gba {
    pub ctx: Sdl,

    pub canvas: Canvas<Window>,

    cpu: Cpu<GbaMmu>,
    mmu: GbaMmu,
    ppu: Ppu,
}

impl Gba {
    pub fn new(rom: GameRom) -> Box<Gba> {
        unsafe {
            let mut gba: Box<Gba> = Box::new(mem::uninitialized());

            ptr::write(&mut gba.ctx, sdl2::init().unwrap());
            let video = gba.ctx.video().unwrap();
            let window = video
                .window("GBA", 480, 320)
                .position_centered()
                .build()
                .unwrap();

            ptr::write(&mut gba.canvas, window.into_canvas().build().unwrap());
            gba.canvas.set_logical_size(240, 160).unwrap();

            ptr::write(&mut gba.mmu, GbaMmu::new_with_rom(rom));

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
                    Shared::new(&mut gba.canvas),
                    Shared::empty(),
                    Shared::new(&mut gba.mmu.vram),
                ),
            );

            gba
        }
    }

    pub fn run(&self) -> Result<()> {
        let mut event_pump = self.ctx.event_pump().unwrap();
        loop {

            event_pump.pump_events();
            let keys = event_pump.keyboard_state();
            if keys.is_scancode_pressed(sdl2::keyboard::Scancode::Escape) {
                break;
            }

            thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
        Ok(())
    }
}
