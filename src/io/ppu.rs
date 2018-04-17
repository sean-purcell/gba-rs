use std::mem;

use sdl2::render::Texture;

use bit_util::extract;

use mmu::Mmu;
use mmu::ram::Ram;
use shared::Shared;

use super::IoReg;

#[derive(Clone, Copy, Debug)]
struct RotScale {
    xref: u32,
    yref: u32,

    dx: u16,
    dmx: u16,
    dy: u16,
    dmy: u16,
}

const PIX_BYTES: usize = 3;
const ROW_BYTES: usize = PIX_BYTES * 240;
const FRAME_BYTES: usize = ROW_BYTES * 160;

/// Handle scanline drawing here
pub struct Ppu<'a> {
    texture: Shared<Texture<'a>>,

    pixels: [u8; FRAME_BYTES],

    io: Shared<IoReg>,
    vram: Shared<Ram>,
    col: u32,
    row: u32,
    delay: u8,

}

impl<'a> Ppu<'a> {
    pub fn new(texture: Shared<Texture<'a>>, io: Shared<IoReg>, vram: Shared<Ram>) -> Ppu<'a> {
        Ppu {
            texture: texture,
            pixels: [0u8; FRAME_BYTES],
            io: io,
            vram: vram,
            col: 0,
            row: 0,
            delay: 0,
        }
    }

    pub fn cycle(&mut self) {
        if self.delay != 0 {
            self.delay -= 1;
            return;
        }
        // Want to render pixel at col, row
        let dspcnt = self.io.get_priv(0x000000) as u32;

        if self.col < 240 && self.row < 160 {
            for i in 0..3 {
                self.pixels[self.col as usize * PIX_BYTES + self.row as usize * ROW_BYTES + i] = 255;
            }
        }
        match extract(dspcnt, 0, 3) {
            // mode
            3 => {
            }
            6 | 7 => warn!("invalid mode"),
            _ => {},
        }

        self.delay = 3;
        self.col += 1;
        if self.col == 308 {
            self.col = 0;
            self.row += 1;

            if self.row == 228 {
                // wrap around, blit our image to the texture
                self.texture.update(None, &self.pixels, ROW_BYTES).unwrap();

                self.row = 0;
            }
        }
    }
}
