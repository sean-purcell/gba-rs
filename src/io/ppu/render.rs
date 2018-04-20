use byteorder::{ByteOrder, LittleEndian};

use bit_util::{bit, extract, sign_extend};
use mmu::Mmu;

use super::{Ppu, COLS, ROWS, PIX_BYTES};

type Colour = (u8, u8, u8);

impl<'a> Ppu<'a> {
    /// Renders the current line into the line field in state
    pub(super) fn render_line(&mut self, row: u32) {
        let dspcnt = self.io.get_priv(0);
        let mode = extract(dspcnt as u32, 0, 3);
        match mode {
            0 | 1 | 2 | 4 | 5 => (), //unimplemented!(),
            3 => self.render_line_mode3(row, dspcnt),
            6 | 7 => warn!("Invalid display mode"),
            _ => unreachable!(),
        };

        for x in 0..COLS {
            let idx = row * COLS + x;
            let off = idx as usize * PIX_BYTES;
            let colour = colour16_rgb(self.state.line[x as usize] as u16);
            let rgb = colour_pack(colour);
            LittleEndian::write_u32(&mut self.pixels[off..off + PIX_BYTES], rgb);
        }
    }

    fn render_line_mode3(&mut self, row: u32, dspcnt: u16) {
        for x in 0..COLS {
            let idx = row * COLS + x;
            self.state.line[x as usize] = self.mmu.vram.load16(idx * 2) as u32;
            if self.state.line[x as usize] != 0 {
                error!("{} {} {:#x}", x, row, self.state.line[x as usize]);
            }
        }
    }

    /*
    fn compute_colour(&self, x: u32, y: u32) -> Colour {
        // Want to render pixel at col, row
        let dspcnt = self.io.get_priv(0x000000) as u32;

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
    */
}

pub(super) struct RenderState {
    // Stealing a trick from VBA: upper bits are priority
    line0: [u32; COLS as usize],
    line1: [u32; COLS as usize],
    line2: [u32; COLS as usize],
    line3: [u32; COLS as usize],
    lineo: [u32; COLS as usize],
    line_objwindow: [u32; COLS as usize],

    line: [u32; COLS as usize],
}

impl Default for RenderState {
    fn default() -> Self {
        RenderState {
            line0: [0; COLS as usize],
            line1: [0; COLS as usize],
            line2: [0; COLS as usize],
            line3: [0; COLS as usize],
            lineo: [0; COLS as usize],
            line_objwindow: [0; COLS as usize],

            line: [0; COLS as usize],
        }
    }
}

enum Layer {
    Bg0,
    Bg1,
    Bg2,
    Bg3,
    Obj,
}

fn colour16_rgb(colour: u16) -> (u8, u8, u8) {
    let c32 = colour as u32;
    (
        (extract(c32, 0, 5) << 3) as u8,
        (extract(c32, 5, 5) << 3) as u8,
        (extract(c32, 10, 5) << 3) as u8,
    )
}

fn colour_pack(colour: Colour) -> u32 {
    ((colour.0 as u32) << 16 | (colour.1 as u32) << 8 | (colour.2 as u32))
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
