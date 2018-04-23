use std::cmp::min;

use super::*;

use mmu::gba::Gba as GbaMmu;

trait Dspcnt {
    fn layout2d(self) -> bool;
    fn objwin_enable(self) -> bool;
    fn mode(self) -> u32;
}

impl Dspcnt for u16 {
    #[inline]
    fn layout2d(self) -> bool {
        bit(self as u32, 6) == 0
    }

    #[inline]
    fn objwin_enable(self) -> bool {
        bit(self as u32, 15) == 1
    }

    #[inline]
    fn mode(self) -> u32 {
        extract(self as u32, 0, 3)
    }
}

pub(super) fn render_obj_line(
    line: &mut LineBuf,
    owin: &mut LineBuf,
    row: u32,
    mmu: &GbaMmu,
    dspcnt: u16,
) {
    for x in 0..240 {
        line[x as usize] = TRANSPARENT;
        owin[x as usize] = 0;
    }

    let objwin = dspcnt.objwin_enable();

    // 128 objects
    for o in 0..128 {
        let a0 = mmu.oam.load16(o * 8 + 0) as u32;
        if extract(a0, 8, 2) == 2 || extract(a0, 10, 2) == 3 {
            // disabled
            continue
        }

        let a1 = mmu.oam.load16(o * 8 + 2) as u32;
        let a2 = mmu.oam.load16(o * 8 + 4) as u32;

        let (xsize, ysize) = match (extract(a0, 14, 2), extract(a1, 14, 2)) {
            (0, x) if x < 4 => (8 * (1 << x), 8 * (1 << x)),
            (1, 0) => (16, 8),
            (1, 1) => (32, 8),
            (1, 2) => (32, 16),
            (1, 3) => (64, 32),
            (2, 0) => (8, 16),
            (2, 1) => (8, 32),
            (2, 2) => (16, 32),
            (2, 3) => (32, 64),
            (3, x) if x < 4 => /* invalid */ (0, 0),
            (_, _) => unreachable!(),
        };

        // don't do scaling for now
        // FIXME: if y is large this should wrap
        let y0 = extract(a0, 0, 8);

        if !(y0 <= row && row < y0 + ysize) &&
            !(y0 <= row + 256 && row + 256 < y0 + ysize) {
            continue
        }

        let iy = (256 + row - y0) % 256;
        let ty = if bit(a1, 13) == 1 {
            ysize - 1 - iy
        } else {
            iy
        };

        // if 2d layout mode is enabled, bottom bit of tile is ignored
        let (tbase, row_inc) = if dspcnt.layout2d() {
            (extract(a2, 0, 10), 32)
        } else {
            (extract(a2, 0, 10), xsize / 8)
        };
        if dspcnt.mode() > 2 && tbase < 512 {
            continue
        };

        let prio = extract(a2, 10, 2) << 28 | if extract(a0, 10, 2) == 1 {
            1 << 16 /* semi-transparent */
        } else { 0 };

        let palette_mode = bit(a0, 13);
        let palette = (1-palette_mode) * extract(a2, 12, 4);
        // 1 if 16/16, 2 if 256/1
        let col_inc = palette_mode + 1;

        // FIXME: this should also wrap
        let x0 = extract(a1, 0, 9);

        let xflip = bit(a1, 12) == 1;
        for x in x0..x0+xsize {
            let sx = x % 512;
            if sx >= 240 {
                continue
            }
            let ix = x - x0;
            let tx = if xflip {
                xsize - 1 - ix
            } else {
                ix
            };

            if line[sx as usize] != TRANSPARENT {
                // another tile already wrote here
                continue
            }

            let t = (tbase + (tx / 8) * col_inc + (ty / 8) * row_inc) & 1023;
            let idx = (tx % 8) + (ty % 8) * 8;

            let tile_addr = 0x10000 + t * 32;
            let palette_colour = if palette_mode == 0 {
                let v = mmu.vram.load8(tile_addr + idx / 2);
                (v >> ((idx&1) * 4)) & 0xf
            } else {
                mmu.vram.load8(tile_addr + idx)
            };

            if palette_colour == 0 {
                continue
            }

            // if in window mode
            if extract(a0, 10, 2) == 2 {
                owin[sx as usize] = 1;
            } else {
                let colour = mmu.pram.load16(
                    0x200 + palette * 32 + (palette_colour as u32) * 2);
                line[sx as usize] = (colour as u32) | prio;
            }
        }
    }
}
