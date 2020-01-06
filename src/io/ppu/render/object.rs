use super::*;

use mmu::gba::Gba as GbaMmu;
use mmu::Mmu;

pub(super) const SEMITRANS: u32 = 1 << 16;

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
        let a0 = mmu.oam.load16(o * 8 + 0).get() as u32;
        if extract(a0, 8, 2) == 2 || extract(a0, 10, 2) == 3 {
            // disabled
            continue;
        }

        let a1 = mmu.oam.load16(o * 8 + 2).get() as u32;
        let a2 = mmu.oam.load16(o * 8 + 4).get() as u32;

        let (xsize, ysize): (u32, u32) = match (extract(a0, 14, 2), extract(a1, 14, 2)) {
            (0, x) if x < 4 => (8 * (1 << x), 8 * (1 << x)),
            (1, 0) => (16, 8),
            (1, 1) => (32, 8),
            (1, 2) => (32, 16),
            (1, 3) => (64, 32),
            (2, 0) => (8, 16),
            (2, 1) => (8, 32),
            (2, 2) => (16, 32),
            (2, 3) => (32, 64),
            (3, x) if x < 4 =>
            /* invalid */
            {
                (0, 0)
            }
            (_, _) => unreachable!(),
        };

        let y0 = extract(a0, 0, 8);

        let (xarea, yarea) = if extract(a0, 8, 2) == 3 {
            (xsize * 2, ysize * 2)
        } else {
            (xsize, ysize)
        };

        // TODO: Implement objects rendering at the top and not the bottom if it would
        // wrap around
        let iy = (256 + row - y0) % 256;
        if iy >= yarea {
            continue;
        }

        let (mut xval, mut yval, dx, dy) = if bit(a0, 8) == 1 {
            // instead of based around top-left, it is based around centre
            // q: screen coords, p: texture coords
            // p = Q * (q - q0) + p0
            let param_idx = extract(a1, 9, 5);
            let rparams = RotScaleParams::new(
                mmu.oam.load16(0x06 + 0x20 * param_idx).get(),
                mmu.oam.load16(0x0E + 0x20 * param_idx).get(),
                mmu.oam.load16(0x16 + 0x20 * param_idx).get(),
                mmu.oam.load16(0x1E + 0x20 * param_idx).get(),
            );

            // p0_z = (zsize << 8) / 2
            let xval = (xsize << 7)
                .wrapping_sub((xarea / 2).wrapping_mul(rparams.a))
                .wrapping_sub((yarea / 2).wrapping_mul(rparams.b))
                .wrapping_add(iy.wrapping_mul(rparams.b));
            let yval = (ysize << 7)
                .wrapping_sub((xarea / 2).wrapping_mul(rparams.c))
                .wrapping_sub((yarea / 2).wrapping_mul(rparams.d))
                .wrapping_add(iy.wrapping_mul(rparams.d));
            (xval, yval, rparams.a, rparams.c)
        } else {
            // create relevant values for horizontal/vertical flip
            let (xval, dx) = if bit(a1, 12) == 0 {
                (0, 0x0100)
            } else {
                ((xsize - 1) << 8, 0xffffff00)
            };
            let yval = if bit(a1, 13) == 0 { iy } else { ysize - 1 - iy } << 8;
            (xval, yval, dx, 0)
        };

        let palette_mode = bit(a0, 13);
        let is_win = extract(a0, 10, 2) == 2;

        // if 2d layout mode is enabled, bottom bit of tile is ignored
        let (tbase, row_inc) = if dspcnt.layout2d() {
            (extract(a2, 0, 10), 32)
        } else {
            // divide by 8 if 16/16, otherwise divide by 4
            (extract(a2, 0, 10), xsize / (4 * (2 - palette_mode)))
        };
        if dspcnt.mode() > 2 && tbase < 512 {
            continue;
        };

        let prio = (extract(a2, 10, 2) << 28)
            | (o << 20)
            | if extract(a0, 10, 2) == 1 {
                SEMITRANS
            } else {
                0
            };

        let palette = (1 - palette_mode) * extract(a2, 12, 4);
        // 1 if 16/16, 2 if 256/1
        let col_inc = palette_mode + 1;

        let x0 = extract(a1, 0, 9);
        for x in x0..x0 + xarea {
            let sx = x % 512;

            let (tx, ty) = (xval >> 8, yval >> 8);
            xval = xval.wrapping_add(dx);
            yval = yval.wrapping_add(dy);

            if sx >= 240
                || (!is_win && line[sx as usize] < prio)
                || (is_win && owin[sx as usize] != 0)
            {
                continue;
            }
            if tx >= xsize || ty >= ysize {
                continue;
            }

            let t = (tbase + (tx / 8) * col_inc + (ty / 8) * row_inc) & 1023;
            let idx = (tx % 8) + (ty % 8) * 8;

            let tile_addr = 0x10000 + t * 32;
            let palette_colour = if palette_mode == 0 {
                let v = mmu.vram.load8(tile_addr + idx / 2).get();
                (v >> ((idx & 1) * 4)) & 0xf
            } else {
                mmu.vram.load8(tile_addr + idx).get()
            };

            if palette_colour == 0 {
                continue;
            }

            // if in window mode
            if is_win {
                if objwin {
                    owin[sx as usize] = 1;
                }
            } else {
                let colour = mmu
                    .pram
                    .load16(0x200 + palette * 32 + (palette_colour as u32) * 2)
                    .get();
                line[sx as usize] = (colour as u32) | prio;
            }
        }
    }
}
