use std::default::Default;

use sdl2::render::Texture;
use serde::{Serialize, Serializer, Deserialize};

use mmu::gba::Gba as GbaMmu;
use shared::Shared;

use super::*;
use super::dma::Trigger;

mod render;

pub const COLS: u32 = 240;
pub const ROWS: u32 = 160;

const PIX_BYTES: usize = 4;
const ROW_BYTES: usize = PIX_BYTES * (COLS as usize);
const FRAME_BYTES: usize = ROW_BYTES * (ROWS as usize);

/// Handle scanline drawing here
pub struct Ppu<'a> {
    texture: Shared<Texture<'a>>,

    pixels: [u8; FRAME_BYTES],

    io: Shared<IoReg<'a>>,
    mmu: Shared<GbaMmu<'a>>,
    col: u32,
    row: u32,
    delay: u8,

    state: render::RenderState,
}

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
            state: Default::default(),
        }
    }

    pub fn cycle(&mut self) {
        if self.delay != 0 {
            self.delay -= 1;
            return;
        }

        self.delay = 3;

        if self.col == 0 {
            if self.row == 0 {
                self.frame_start();
            }
            self.line_start();
        }

        self.col += 1;
        if self.col == 240 {
            self.hblank();
        } else if self.col == 308 {
            self.col = 0;
            self.row += 1;

            if self.row == 160 {
                self.vblank();
            } else if self.row == 228 {
                self.row = 0;
                self.vblank_end();
            }
        }
    }

    fn frame_start(&mut self) {
        self.update_bg2ref();
        self.update_bg3ref();

        let mut ds = self.io.get_priv(DISPSTAT);
        ds &= !1; // unset vblank flag
        self.io.set_priv(DISPSTAT, ds);
    }

    fn line_start(&mut self) {
        self.io.set_priv(VCOUNT, self.row as u16);
        let mut ds = self.io.get_priv(DISPSTAT);
        if (ds >> 8) == self.row as u16 {
            ds |= 4; // vcounter
            if ds & 0x20 != 0 {
                self.io.raise_interrupt(2);
            }
        } else {
            // unset vcounter flag
            ds &= !4;
        }
        ds &= !2; // unset hblank flag
        self.io.set_priv(DISPSTAT, ds);

        if self.row < 160 {
            // The borrow checker is really strict... self.row.clone() didn't work
            let row = self.row;
            self.render_line(row);
        }
    }

    fn hblank(&mut self) {
        let mut ds = self.io.get_priv(DISPSTAT);
        ds |= 2;
        if ds & 0x10 != 0 {
            self.io.raise_interrupt(1);
        }
        self.io.set_priv(DISPSTAT, ds);
        if self.row < 160 {
            self.io.dma.trigger(Trigger::HBlank);
        }
    }

    fn vblank(&mut self) {
        let mut ds = self.io.get_priv(DISPSTAT);
        ds |= 1;
        if ds & 0x8 != 0 {
            self.io.raise_interrupt(0);
        }
        self.io.set_priv(DISPSTAT, ds);
        self.io.dma.trigger(Trigger::VBlank);
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

    pub fn update_bg2ref(&mut self) {
        let xl = self.io.get_priv(0x28);
        let xh = self.io.get_priv(0x2a);
        let yl = self.io.get_priv(0x2c);
        let yh = self.io.get_priv(0x2e);

        self.state.bg2ref = render::BgRef::new(xl, xh, yl, yh);
    }

    pub fn update_bg3ref(&mut self) {
        let xl = self.io.get_priv(0x38);
        let xh = self.io.get_priv(0x3a);
        let yl = self.io.get_priv(0x3c);
        let yh = self.io.get_priv(0x3e);

        self.state.bg3ref = render::BgRef::new(xl, xh, yl, yh);
    }
}

impl<'a> Serialize for Ppu<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Don't serialize the pixels, most of the time save states will be at
        // a frame boundary anyway
        let mut s = serializer.serialize_struct("gba_rs::io::ppu::render::Ppu", 4)?;
        s.serialize_field("col", &self.col)?;
        s.serialize_field("row", &self.row)?;
        s.serialize_field("delay", &self.delay)?;
        s.serialize_field("state", &self.state)?;
        s.end()
    }
}
