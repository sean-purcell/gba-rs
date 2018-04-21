use super::*;

use mmu::gba::Gba as GbaMmu;

#[derive(Default, Copy, Clone, Debug)]
pub struct RotScaleParams {
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

pub enum BgControl {
    TileMap(u16),
    Bitmap(u16),
}

impl BgControl {
    #[inline]
    fn base_addr(&self) -> u32 {
        use self::BgControl::*;
        match *self {
            TileMap(x) => unimplemented!(),
            Bitmap(dspcnt) => {
                let d = dspcnt as u32;
                if extract(d, 0, 3) == 3 || bit(d, 4) == 0 {
                    0x0
                } else {
                    0xA000
                }
            }
        }
    }

    #[inline]
    fn priority(&self) -> u32 {
        use self::BgControl::*;
        match *self {
            TileMap(ctrl) => extract(ctrl as u32, 0, 2),
            Bitmap(dspcnt) => 0,
        }
    }

    #[inline]
    fn size(&self) -> (u32, u32) {
        use self::BgControl::*;
        match *self {
            TileMap(ctrl) => {
                let s = 128 * (1 << extract(ctrl as u32, 14, 2));
                (s, s)
            }
            Bitmap(dspcnt) => {
                match extract(dspcnt as u32, 0, 3) {
                    3 => (240, 160),
                    4 => (240, 160),
                    5 => (160, 128),
                    _ => unreachable!(),
                }
            }
        }
    }

    #[inline]
    fn wrap(&self) -> bool {
        use self::BgControl::*;
        match *self {
            TileMap(ctrl) => bit(ctrl as u32, 13) == 1,
            Bitmap(dspcnt) => false,
        }
    }
}

// FIXME: make this function handle palettes
pub(super) fn render_rotscale_line(
    line: &mut LineBuf,
    row: u32,
    mmu: &GbaMmu,
    bgref: &mut BgRef,
    params: RotScaleParams,
    ctrl: BgControl,
) {
    let prio = ctrl.priority() << 28;

    let base = ctrl.base_addr();

    let (xsize, ysize) = ctrl.size();

    let mut xval = bgref.xref;
    let mut yval = bgref.yref;
    for x in 0..COLS {
        let nx = xval >> 8;
        let ny = yval >> 8;

        let colour = if ctrl.wrap() {
            let ix = nx % xsize;
            let iy = ny % ysize;
            (mmu.vram.load16((iy * xsize + ix) * 2) as u32) | prio
        } else {
            // unsigned
            if nx < xsize && ny < ysize {
                (mmu.vram.load16((ny * xsize + nx) * 2) as u32) | prio
            } else {
                TRANSPARENT
            }
        };
        line[x as usize] = colour;

        xval = xval.wrapping_add(params.a);
        yval = yval.wrapping_add(params.c);
    }

    bgref.xref = bgref.xref.wrapping_add(params.b);
    bgref.yref = bgref.yref.wrapping_add(params.d);
}
