use sdl2::render::Canvas;
use sdl2::video::Window;

use mmu::Mmu;
use mmu::ram::Ram;
use shared::Shared;

use super::IoReg;

/// Handle scanline drawing here
pub struct Ppu {
    canvas: Shared<Canvas<Window>>,
    io: Shared<IoReg>,
    vram: Shared<Ram>,
    col: u32,
    row: u32,
}

impl Ppu {
    pub fn new(canvas: Shared<Canvas<Window>>, io: Shared<IoReg>, vram: Shared<Ram>) -> Ppu {
        Ppu {
            canvas: canvas,
            io: io,
            vram: vram,
            col: 0,
            row: 0,
        }
    }

    pub fn cycle(&mut self) {
        // Want to render pixel at col, row
        println!("{:?}", self.vram.load32(0));
    }
}
