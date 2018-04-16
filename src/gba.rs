use sdl2;
use sdl2::Sdl;
use sdl2::render::Canvas;
use sdl2::video::Window;

use cpu::Cpu;
use io::ppu::Ppu;
use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;

/// Parent container for all components of the system
pub struct Gba {
    pub ctx: Sdl,

    pub canvas: Canvas<Window>,

        /*
    cpu: Cpu,
    mmu: GbaMmu,
    ppu: Ppu, */
}

impl Gba {
    pub fn new() -> Gba {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        let window = video
            .window("GBA", 480, 320)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_logical_size(240, 160);

        Gba {
            ctx: sdl,
            canvas: canvas,
        }
    }
}
