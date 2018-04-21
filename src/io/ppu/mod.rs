use std::default::Default;

use sdl2::render::Texture;

use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;
use shared::Shared;

use super::IoReg;

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

        if self.col == 0 && self.row < 160 {
            // The borrow checker is really strict... self.row.clone() didn't work
            let row = self.row;
            self.render_line(row);
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

        // TODO: this is one cycle off from when it should actually happen probably
        self.update_bg2ref();
        self.update_bg3ref();
    }

    fn hblank_end(&mut self) {
        self.io.set_priv(0x6, self.row as u16);
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
