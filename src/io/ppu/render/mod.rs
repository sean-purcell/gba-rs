// Unimplemented rendering features:
// Fast access mode
// Mosaic
// Colour mixing
// Sprite affine transforms

use std::ops::{Deref, DerefMut};

use byteorder::{ByteOrder, LittleEndian};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use bit_util::{bit, extract, sign_extend};

use super::{Ppu, COLS, DSPCNT, PIX_BYTES};

mod background;
mod combine;
mod object;

const TRANSPARENT: u32 = 0xf0000000;

impl<'a> Ppu<'a> {
    /// Renders the current line into the line field in state
    pub(super) fn render_line(&mut self, row: u32) {
        let dspcnt = self.io.get_priv(DSPCNT);
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
}

struct LineBuf([u32; COLS as usize]);

impl Default for LineBuf {
    fn default() -> Self {
        LineBuf([0; COLS as usize])
    }
}

impl Deref for LineBuf {
    type Target = [u32];
    fn deref(&self) -> &[u32] {
        &self.0
    }
}

impl DerefMut for LineBuf {
    fn deref_mut(&mut self) -> &mut [u32] {
        &mut self.0
    }
}

#[derive(Default)]
pub(super) struct RenderState {
    // Stealing a trick from VBA: upper bits are priority
    // TODO: linebufs are just a memory allocation thing,
    // check if they're really necessary
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

#[derive(Default, Copy, Clone, Debug, Serialize, Deserialize)]
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
    (y1 == y2 && y1 >= 0xe8)
        || if y1 <= y2 {
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

impl Serialize for RenderState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serializing LineBuf's is pointless, they're just there to avoid
        // allocating memory on the stack repeatedly
        let mut s = serializer.serialize_struct("gba_rs::io::ppu::render::Ppu", 2)?;
        s.serialize_field("bg2ref", &self.bg2ref)?;
        s.serialize_field("bg3ref", &self.bg3ref)?;
        s.end()
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
