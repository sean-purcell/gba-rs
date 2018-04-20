use byteorder::{ByteOrder, LittleEndian};
use sdl2::render::Texture;

use bit_util::{bit, extract, sign_extend};

use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;
use mmu::ram::Ram;
use shared::Shared;

use super::IoReg;

const PIX_BYTES: usize = 4;
const ROW_BYTES: usize = PIX_BYTES * 240;
const FRAME_BYTES: usize = ROW_BYTES * 160;

/// Handle scanline drawing here
pub struct Ppu<'a> {
    texture: Shared<Texture<'a>>,

    pixels: [u8; FRAME_BYTES],

    io: Shared<IoReg<'a>>,
    mmu: Shared<GbaMmu<'a>>,
    col: u32,
    row: u32,
    delay: u8,
}

type Colour = (u8, u8, u8);

impl<'a> Ppu<'a> {
    pub fn new(
        texture: Shared<Texture<'a>>,
        io: Shared<IoReg<'a>>,
        mmu: Shared<GbaMmu<'a>>,
    ) -> Self {
        Ppu {
            texture: texture,
            pixels: [0u8; FRAME_BYTES],
            io: io,
            mmu: mmu,
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

        if self.col < 240 && self.row < 160 {
            let idx = (self.row * 240 + self.col) as usize * PIX_BYTES;

            let (r, g, b) = self.compute_colour(self.col, self.row);

            LittleEndian::write_u32(
                &mut self.pixels[idx..idx + PIX_BYTES],
                (b as u32) | ((g as u32) << 8) | ((r as u32) << 16),
            );
        }

        self.delay = 3;
        self.col += 1;
        if self.col == 308 {
            self.col = 0;
            self.row += 1;
            self.hblank_end();

            if self.row == 228 {
                self.row = 0;
                self.vblank_end();
            }
        }
    }

    fn compute_colour(&self, x: u32, y: u32) -> Colour {
        // Want to render pixel at col, row
        let dspcnt = self.io.get_priv(0x000000) as u32;

        let (win0, win1, wino) = (bit(dspcnt, 13), bit(dspcnt, 14), bit(dspcnt, 15));

        // compute background colour
        let bg = match extract(dspcnt, 0, 3) {
            // mode
            3 => self.get_colour_bg2(x, y),
            6 | 7 => {
                warn!("invalid mode");
                None
            }
            _ => None,
        };
        match bg {
            Some(c) => c,
            None => (0, 0, 0),
        }
    }

    fn get_colour_bg2(&self, x: u32, y: u32) -> Option<Colour> {
        let (nx, ny) = self.compute_scale(x, y, Layer::Bg2);

        // FIXME: replace with layer size
        if nx < 240 && ny < 160 {
            let idx = ny * 240 + nx;
            // FIXME: do paletting
            let colour = self.mmu.vram.load16(idx * 2);

            Some(colour16_rgb(colour))
        } else {
            None
        }
    }

    #[inline]
    fn compute_scale(&self, x: u32, y: u32, bg: Layer) -> (u32, u32) {
        let base = match bg {
            Layer::Bg2 => 0x20,
            Layer::Bg3 => 0x30,
            _ => unreachable!(),
        };

        let xref = sign_extend(
            (self.io.get_priv(base + 0x8) as u32) |
                ((self.io.get_priv(base + 0xa) as u32) << 16),
            28,
        );
        let yref = sign_extend(
            (self.io.get_priv(base + 0xc) as u32) |
                ((self.io.get_priv(base + 0xe) as u32) << 16),
            28,
        );
        let a = self.io.get_priv(base + 0x0) as i16 as i32 as u32;
        let b = self.io.get_priv(base + 0x2) as i16 as i32 as u32;
        let c = self.io.get_priv(base + 0x4) as i16 as i32 as u32;
        let d = self.io.get_priv(base + 0x6) as i16 as i32 as u32;

        let dx = (x << 8).wrapping_sub(xref);
        let dy = (y << 8).wrapping_sub(yref);
        (
            xref.wrapping_add(dx.wrapping_mul(a)).wrapping_add(
                dy.wrapping_mul(b),
            ) >> 16,
            yref.wrapping_add(dx.wrapping_mul(c)).wrapping_add(
                dy.wrapping_mul(d),
            ) >> 16,
        )
    }

    fn vblank_end(&mut self) {
        // wrap around, blit our image to the texture
        let pixels = Shared::new(&mut self.pixels);
        self.texture
            .with_lock(None, |buf, pitch| for row in 0..160 {
                let buf_start = row * pitch;
                let pix_start = row * ROW_BYTES;
                buf[buf_start..buf_start + ROW_BYTES].clone_from_slice(
                    &pixels
                        [pix_start..pix_start + ROW_BYTES],
                );
            })
            .unwrap();
    }

    fn hblank_end(&mut self) {
        self.io.set_priv(0x6, self.row as u16);
    }

    pub fn update_bg2ref(&mut self) {}

    pub fn update_bg3ref(&mut self) {}
}

fn colour16_rgb(colour: u16) -> (u8, u8, u8) {
    let c32 = colour as u32;
    (
        (extract(c32, 0, 5) << 3) as u8,
        (extract(c32, 5, 5) << 3) as u8,
        (extract(c32, 10, 5) << 3) as u8,
    )
}

enum Layer {
    Bg0,
    Bg1,
    Bg2,
    Bg3,
    Obj,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_colourconvert() {
        assert_eq!((0xf8, 0, 0), colour16_rgb(0x1f));
        assert_eq!((0, 0xf8, 0), colour16_rgb(0x3e0));
        assert_eq!((0, 0, 0xf8), colour16_rgb(0x7c00));
    }
}
