// Unimplemented rendering features:
// Fast access mode
// Mosaic
// Colour mixing
// Sprite affine transforms

use std::ops::{Deref, DerefMut};

use byteorder::{ByteOrder, LittleEndian};

use bit_util::{bit, extract, sign_extend};
use mmu::Mmu;

use super::{Ppu, COLS, ROWS, PIX_BYTES};

mod background;
mod object;
mod combine;

const TRANSPARENT: u32 = 0xf0000000;

impl<'a> Ppu<'a> {
    /// Renders the current line into the line field in state
    pub(super) fn render_line(&mut self, row: u32) {
        let dspcnt = self.io.get_priv(0);
        let mode = extract(dspcnt as u32, 0, 3);
        debug!("Rendering mode {} scanline: {:#06x}", mode, dspcnt);
        self.combine_line(row, dspcnt);

        for x in 0..COLS {
            let idx = row * COLS + x;
            let off = idx as usize * PIX_BYTES;
            let colour = colour16_rgb(self.state.line[x as usize] as u16);
            let rgb = colour_pack(colour);
            LittleEndian::write_u32(&mut self.pixels[off..off + PIX_BYTES], rgb);
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

struct LineBuf {
    a: [u32; COLS as usize],
}

impl Default for LineBuf {
    fn default() -> Self {
        LineBuf { a: [0; COLS as usize] }
    }
}

impl Deref for LineBuf {
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        &self.a
    }
}

impl DerefMut for LineBuf {
    fn deref_mut(&mut self) -> &mut [u32] {
        &mut self.a
    }
}

#[derive(Default)]
pub(super) struct RenderState {
    // Stealing a trick from VBA: upper bits are priority
    line0: LineBuf,
    line1: LineBuf,
    line2: LineBuf,
    line3: LineBuf,
    lineo: LineBuf,
    line_objwindow: LineBuf,

    line: LineBuf,

    pub(super) bg2ref: BgRef,
    pub(super) bg3ref: BgRef,
}

#[derive(Default, Copy, Clone, Debug)]
pub struct BgRef {
    xref: u32,
    yref: u32,
}

#[derive(Default, Copy, Clone, Debug)]
struct RotScaleParams {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl RotScaleParams {
    pub fn new(a: u16, b: u16, c: u16, d: u16) -> Self {
        Self {
            a: a as i16 as u32,
            b: b as i16 as u32,
            c: c as i16 as u32,
            d: d as i16 as u32,
        }
    }
}

impl BgRef {
    pub(super) fn new(xl: u16, xh: u16, yl: u16, yh: u16) -> Self {
        Self {
            xref: sign_extend((xl as u32) | ((xh as u32) << 16), 28),
            yref: sign_extend((yl as u32) | ((yh as u32) << 16), 28),
        }
    }
}

fn colour16_rgb(colour: u16) -> (u8, u8, u8) {
    let c32 = colour as u32;
    (
        (extract(c32, 0, 5) << 3) as u8,
        (extract(c32, 5, 5) << 3) as u8,
        (extract(c32, 10, 5) << 3) as u8,
    )
}

fn colour_pack(colour: (u8, u8, u8)) -> u32 {
    ((colour.0 as u32) << 16 | (colour.1 as u32) << 8 | (colour.2 as u32))
}

fn in_win_vert(winv: u16, row: u32) -> bool {
    let y1 = (winv >> 8) as u32;
    let y2 = (winv & 0xff) as u32;

    // logic here is weird, copying from vba
    (y1 == y2 && y1 >= 0xe8) ||
        if y1 <= y2 {
            row >= y1 && row < y2
        } else {
            row >= y1 || row < y2
        }
}

fn in_win_hori(winh: u16, col: u32) -> bool {
    let x1 = (winh >> 8) as u32;
    let x2 = (winh & 0xff) as u32;

    if x1 <= x2 {
        col >= x1 && col < x2
    } else {
        col >= x1 || col < x2
    }
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
