use super::*;

use mmu::gba::Gba as GbaMmu;
use mmu::Mmu;

pub enum RotScaleCtrl {
    TileMap(u16),
    Bitmap(u16),
}

impl RotScaleCtrl {
    #[inline]
    fn base_addr(&self) -> u32 {
        use self::RotScaleCtrl::*;
        match *self {
            TileMap(ctrl) => extract(ctrl as u32, 8, 5) * 2 * 1024,
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
    fn tile_base_addr(&self) -> u32 {
        use self::RotScaleCtrl::*;
        match *self {
            TileMap(ctrl) => extract(ctrl as u32, 2, 2) * 16 * 1024,
            Bitmap(_dspcnt) => 0,
        }
    }

    #[inline]
    fn priority(&self) -> u32 {
        use self::RotScaleCtrl::*;
        match *self {
            TileMap(ctrl) => extract(ctrl as u32, 0, 2),
            Bitmap(_dspcnt) => 3,
        }
    }

    #[inline]
    fn size(&self) -> (u32, u32) {
        use self::RotScaleCtrl::*;
        match *self {
            TileMap(ctrl) => {
                let s = 128 * (1 << extract(ctrl as u32, 14, 2));
                (s, s)
            }
            Bitmap(dspcnt) => match extract(dspcnt as u32, 0, 3) {
                3 => (240, 160),
                4 => (240, 160),
                5 => (160, 128),
                _ => unreachable!(),
            },
        }
    }

    #[inline]
    fn wrap(&self) -> bool {
        use self::RotScaleCtrl::*;
        match *self {
            TileMap(ctrl) => bit(ctrl as u32, 13) == 1,
            Bitmap(_dspcnt) => false,
        }
    }

    #[inline]
    fn is_palette(&self) -> bool {
        use self::RotScaleCtrl::*;
        match *self {
            TileMap(_ctrl) => true,
            Bitmap(dspcnt) => match extract(dspcnt as u32, 0, 3) {
                3 | 5 => false,
                4 => true,
                _ => unreachable!(),
            },
        }
    }
}

// FIXME: mosaic
pub(super) fn render_rotscale_line(
    line: &mut LineBuf,
    mmu: &GbaMmu,
    bgref: &mut BgRef,
    params: RotScaleParams,
    ctrl: RotScaleCtrl,
    bg: u8,
) {
    // upper 8 bits are priority
    // add 1 so OBJ will have lower priority here
    let prio = (ctrl.priority() << 28) | ((bg as u32 + 1) << 25);

    let base = ctrl.base_addr();
    let tile_base = ctrl.tile_base_addr();

    let (xsize, ysize) = ctrl.size();

    let is_palette = ctrl.is_palette();

    let mut xval = bgref.xref;
    let mut yval = bgref.yref;
    for x in 0..COLS {
        let nx = xval >> 8;
        let ny = yval >> 8;

        let (ix, iy) = if ctrl.wrap() {
            (nx % xsize, ny % ysize)
        } else {
            (nx, ny)
        };

        let colour = if ix < xsize && iy < ysize {
            match ctrl {
                RotScaleCtrl::TileMap(_) => {
                    let tile_idx = (ix / 8) + (iy / 8) * (xsize / 8);
                    let addr = base + tile_idx;
                    let tile = mmu.vram.load8(addr).get() as u32;
                    // 256 colours / 1 palette
                    // one tile is 64 bytes
                    let px_idx = (ix % 8) + (iy % 8) * 8;
                    let colour = mmu.vram.load8(tile_base + tile * 64 + px_idx).get();
                    if colour == 0 {
                        TRANSPARENT
                    } else {
                        mmu.pram.load16(colour as u32 * 2).get() as u32 | prio
                    }
                }
                RotScaleCtrl::Bitmap(_) => {
                    // mode 3/5 are direct colours, 4 is palette
                    let idx = iy * xsize + ix;
                    if is_palette {
                        let colour = mmu.vram.load8(base + idx).get();
                        if colour == 0 {
                            TRANSPARENT
                        } else {
                            mmu.pram.load16(colour as u32 * 2).get() as u32 | prio
                        }
                    } else {
                        mmu.vram.load16(base + idx * 2).get() as u32 | prio
                    }
                }
            }
        } else {
            TRANSPARENT
        };

        line[x as usize] = colour;

        xval = xval.wrapping_add(params.a);
        yval = yval.wrapping_add(params.c);
    }

    bgref.xref = bgref.xref.wrapping_add(params.b);
    bgref.yref = bgref.yref.wrapping_add(params.d);
}

trait TextCtrl {
    fn priority(self) -> u32;
    fn tile_base_addr(self) -> u32;
    fn is256c(self) -> bool;
    fn base_addr(self) -> u32;
    fn size(self) -> (u32, u32);
}

impl TextCtrl for u16 {
    #[inline]
    fn priority(self) -> u32 {
        extract(self as u32, 0, 2)
    }

    #[inline]
    fn tile_base_addr(self) -> u32 {
        extract(self as u32, 2, 2) * 16 * 1024
    }

    #[inline]
    fn is256c(self) -> bool {
        bit(self as u32, 7) == 1
    }

    #[inline]
    fn base_addr(self) -> u32 {
        extract(self as u32, 8, 5) * 2 * 1024
    }

    #[inline]
    fn size(self) -> (u32, u32) {
        match extract(self as u32, 14, 2) {
            0 => (256, 256),
            1 => (512, 256),
            2 => (256, 512),
            3 => (512, 512),
            _ => unreachable!(),
        }
    }
}

pub(super) fn render_textmode_line(line: &mut LineBuf, row: u32, mmu: &GbaMmu, bg: u8) {
    let ctrl = mmu.io.get_priv(8 + (bg as u32) * 2);
    let prio = (ctrl.priority() << 28) | (1 << 27) | ((bg as u32) << 25);

    let base = ctrl.base_addr();
    let tile_base = ctrl.tile_base_addr();

    let (xsize, ysize) = ctrl.size();

    let xoff = extract(mmu.io.get_priv(0x10 + (bg as u32) * 4) as u32, 0, 9);
    let yoff = extract(mmu.io.get_priv(0x12 + (bg as u32) * 4) as u32, 0, 9);

    let c256 = ctrl.is256c();

    for x in 0..COLS {
        let nx = (x + xoff) & (xsize - 1);
        let ny = (row + yoff) & (ysize - 1);

        let map = if xsize == 256 || ysize == 256 {
            (nx >= 256) as u32 + (ny >= 256) as u32
        } else {
            (nx >= 256) as u32 + ((ny >= 256) as u32) * 2
        };

        let ix = nx % 256;
        let iy = ny % 256;

        let tile_idx = (ix / 8) + (iy / 8) * 32;
        let addr = base + map * (2 * 1024) + tile_idx * 2;

        let tile = mmu.vram.load16(addr).get() as u32;

        let palette = if c256 { 0 } else { extract(tile, 12, 4) };
        // FIXME: horizontal flip, vertical flip
        let tile_num = extract(tile, 0, 10);

        // * 1 if c16, * 2 otherwise
        let tx = if bit(tile, 10) == 0 {
            ix % 8
        } else {
            7 - (ix % 8)
        };
        let ty = if bit(tile, 11) == 0 {
            iy % 8
        } else {
            7 - (iy % 8)
        };
        let px_idx = (tx % 8) + (ty % 8) * 8;

        let px_addr = tile_num * 64 + px_idx;
        let palette_colour = if c256 {
            mmu.vram.load8(tile_base + px_addr).get()
        } else {
            (mmu.vram.load8(tile_base + px_addr / 2).get() >> ((px_addr & 1) * 4)) & 0xf
        };
        line[x as usize] = if palette_colour == 0 {
            TRANSPARENT
        } else {
            mmu.pram
                .load16(palette * 32 + (palette_colour as u32) * 2)
                .get() as u32
                | prio
        };
    }
}
