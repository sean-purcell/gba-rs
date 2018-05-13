use std::cmp::{max, min};

use super::*;
use super::background::*;
use super::object::*;

use mmu::Mmu;

#[inline]
fn colour_unpack(c: u16) -> (u8, u8, u8) {
    let c = c as u32;
    (
        extract(c, 0, 5) as u8,
        extract(c, 5, 5) as u8,
        extract(c, 10, 5) as u8,
    )
}

#[inline]
fn colour_repack(c: (u8, u8, u8)) -> u16 {
    (c.0 as u16) | ((c.1 as u16) << 5) | ((c.2 as u16) << 10)
}

#[inline]
fn alpha_blend_component(eva: u32, evb: u32, c1: u8, c2: u8) -> u8 {
    let c1 = c1 as u32;
    let c2 = c2 as u32;
    min(31, (eva * c1 + evb * c2) / 16) as u8
}

#[inline]
fn alpha_blend(bldalpha: u16, c1: u32, c2: u32) -> u32 {
    let eva = extract(bldalpha as u32, 0, 5);
    let evb = extract(bldalpha as u32, 8, 5);

    let c1rgb = colour_unpack(c1 as u16);
    let c2rgb = colour_unpack(c2 as u16);

    colour_repack((
        alpha_blend_component(eva, evb, c1rgb.0, c2rgb.0),
        alpha_blend_component(eva, evb, c1rgb.1, c2rgb.1),
        alpha_blend_component(eva, evb, c1rgb.2, c2rgb.2),
    )) as u32
}

#[inline]
fn brighten_component(evy: u32, c: u8) -> u8 {
    let c = c as u32;
    min(31, c + (evy * (31 - c)) / 16) as u8
}

#[inline]
fn darken_component(evy: u32, c: u8) -> u8 {
    let c = c as i32;
    max(0, c - (evy as i32 * c) / 16) as u8
}

#[inline]
fn brighten(bldy: u16, c: u32) -> u32 {
    let evy = extract(bldy as u32, 0, 5);

    let rgb = colour_unpack(c as u16);
    colour_repack((
        brighten_component(evy, rgb.0),
        brighten_component(evy, rgb.1),
        brighten_component(evy, rgb.2),
    )) as u32
}

#[inline]
fn darken(bldy: u16, c: u32) -> u32 {
    let evy = extract(bldy as u32, 0, 5);

    let rgb = colour_unpack(c as u16);
    colour_repack((
        darken_component(evy, rgb.0),
        darken_component(evy, rgb.1),
        darken_component(evy, rgb.2),
    )) as u32
}

fn blend(
    effect: u32,
    bldcnt: u16,
    bldalpha: u16,
    bldy: u16,
    f: u8,
    fc: u32,
    s: u8,
    sc: u32,
) -> u32 {
    if bit(bldcnt as u32, f) == 1 {
        match effect {
            0 => fc,
            1 => {
                if bit(bldcnt as u32, 8 + s) == 1 {
                    alpha_blend(bldalpha, fc, sc)
                } else {
                    fc
                }
            }
            2 => brighten(bldy, fc),
            3 => darken(bldy, fc),
            _ => unreachable!(),
        }
    } else {
        fc
    }
}

fn blend_semitrans(
    effect: u32,
    bldcnt: u16,
    bldalpha: u16,
    bldy: u16,
    _f: u8,
    fc: u32,
    s: u8,
    sc: u32,
) -> u32 {
    if bit(bldcnt as u32, 8 + s) == 1 {
        alpha_blend(bldalpha, fc, sc)
    } else {
        match effect {
            0 | 1 => fc,
            2 => brighten(bldy, fc),
            3 => darken(bldy, fc),
            _ => unreachable!(),
        }
    }
}

impl<'a> Ppu<'a> {
    fn bg0_drawline(&mut self, mode: u32, row: u32, dspcnt: u16) -> bool {
        let bg0en = mode <= 1 && bit(dspcnt as u32, 8) == 1;
        if bg0en {
            render_textmode_line(&mut self.state.line0, row, &self.mmu, 0);
        }
        bg0en
    }

    fn bg1_drawline(&mut self, mode: u32, row: u32, dspcnt: u16) -> bool {
        let bg1en = mode <= 1 && bit(dspcnt as u32, 9) == 1;
        if bg1en {
            render_textmode_line(&mut self.state.line1, row, &self.mmu, 1);
        }
        bg1en
    }

    fn bg2_drawline(&mut self, mode: u32, row: u32, dspcnt: u16) -> bool {
        let bg2en = bit(dspcnt as u32, 10) == 1;
        if bg2en {
            if mode == 0 {
                render_textmode_line(&mut self.state.line2, row, &self.mmu, 2);
            } else {
                let rparams = RotScaleParams::new(
                    self.io.get_priv(0x20),
                    self.io.get_priv(0x22),
                    self.io.get_priv(0x24),
                    self.io.get_priv(0x26),
                );

                let ctrl = if mode < 3 {
                    RotScaleCtrl::TileMap(self.io.get_priv(0xc))
                } else {
                    RotScaleCtrl::Bitmap(dspcnt)
                };

                render_rotscale_line(
                    &mut self.state.line2,
                    &self.mmu,
                    &mut self.state.bg2ref,
                    rparams,
                    ctrl,
                    2,
                );
            }
        }
        bg2en
    }

    fn bg3_drawline(&mut self, mode: u32, row: u32, dspcnt: u16) -> bool {
        let bg3en = (mode == 0 || mode == 2) && bit(dspcnt as u32, 11) == 1;
        if bg3en {
            if mode == 0 {
                render_textmode_line(&mut self.state.line3, row, &self.mmu, 3);
            } else {
                let rparams = RotScaleParams::new(
                    self.io.get_priv(0x30),
                    self.io.get_priv(0x32),
                    self.io.get_priv(0x34),
                    self.io.get_priv(0x36),
                );

                render_rotscale_line(
                    &mut self.state.line3,
                    &self.mmu,
                    &mut self.state.bg3ref,
                    rparams,
                    RotScaleCtrl::TileMap(self.io.get_priv(0xe)),
                    3,
                );
            }
        }
        bg3en
    }

    fn obj_drawline(&mut self, _mode: u32, row: u32, dspcnt: u16) -> bool {
        let objen = bit(dspcnt as u32, 12) == 1;
        if objen {
            render_obj_line(
                &mut self.state.lineo,
                &mut self.state.line_objwindow,
                row,
                &self.mmu,
                dspcnt,
            );
        }
        objen
    }

    pub(super) fn combine_line(&mut self, row: u32, dspcnt: u16) {
        let mode = extract(dspcnt as u32, 0, 3);

        let bg0en = self.bg0_drawline(mode, row, dspcnt);
        let bg1en = self.bg1_drawline(mode, row, dspcnt);
        let bg2en = self.bg2_drawline(mode, row, dspcnt);
        let bg3en = self.bg3_drawline(mode, row, dspcnt);
        let objen = self.obj_drawline(mode, row, dspcnt);

        let win_enable = extract(dspcnt as u32, 13, 3) != 0;
        let in_win0 = bit(dspcnt as u32, 13) == 1 && in_win_vert(self.io.get_priv(0x44), row);
        let in_win1 = bit(dspcnt as u32, 14) == 1 && in_win_vert(self.io.get_priv(0x46), row);
        let in_wino = bit(dspcnt as u32, 15) == 1;

        let winin = self.io.get_priv(0x48);
        let winout = self.io.get_priv(0x4a);

        let win0h = if in_win0 { self.io.get_priv(0x40) } else { 0 };
        let win1h = if in_win1 { self.io.get_priv(0x42) } else { 0 };

        let bldcnt = self.io.get_priv(0x50);
        let effect = extract(bldcnt as u32, 6, 2);
        let bldalpha = self.io.get_priv(0x52);
        let bldy = self.io.get_priv(0x54);

        let backdrop = (self.mmu.pram.load16(0) as u32) | (0xe << 28);

        for x in 0..COLS {
            let ux = x as usize;
            let en_mask = if win_enable {
                if in_win0 && in_win_hori(win0h, x) {
                    winin & 0xff
                } else if in_win1 && in_win_hori(win1h, x) {
                    winin >> 8
                } else if in_wino && self.state.line_objwindow[ux] != 0 {
                    winout >> 8
                } else {
                    winout & 0xff
                }
            } else {
                0xff
            } as u32;

            let bg0en = bg0en && bit(en_mask, 0) == 1;
            let bg1en = bg1en && bit(en_mask, 1) == 1;
            let bg2en = bg2en && bit(en_mask, 2) == 1;
            let bg3en = bg3en && bit(en_mask, 3) == 1;
            let objen = objen && bit(en_mask, 4) == 1;

            let (first, fc) = {
                let mut fc = backdrop;
                let mut f = 5;
                macro_rules! check {
                    ($c: expr, $i: expr) => {
                        {
                            let val = $c;
                            if val < fc {
                                f = $i;
                                fc = val;
                            }
                        }
                    };
                }
                if objen {
                    check!(self.state.lineo[ux], 4);
                }
                if bg0en {
                    check!(self.state.line0[ux], 0);
                }
                if bg1en {
                    check!(self.state.line1[ux], 1);
                }
                if bg2en {
                    check!(self.state.line2[ux], 2);
                }
                if bg3en {
                    check!(self.state.line3[ux], 3);
                }
                (f, fc)
            };
            let (second, sc) = if (fc & SEMITRANS != 0) || (bit(en_mask, 5) == 1 && effect == 1) {
                let mut sc = backdrop;
                let mut s = 5;
                macro_rules! check {
                    ($c: expr, $i: expr) => {
                        {
                            let val = $c;
                            if val < sc {
                                s = $i;
                                sc = val;
                            }
                        }
                    };
                }
                if objen && first != 4 {
                    check!(self.state.lineo[ux], 4)
                }
                if bg0en && first != 0 {
                    check!(self.state.line0[ux], 0)
                }
                if bg1en && first != 1 {
                    check!(self.state.line1[ux], 1)
                }
                if bg2en && first != 2 {
                    check!(self.state.line2[ux], 2)
                }
                if bg3en && first != 3 {
                    check!(self.state.line3[ux], 3)
                }
                (s, sc)
            } else {
                // bldcnt will be converted to u32,
                // so when we check if the bit is enabled for second target,
                // this will always be a 0 (0 + 16 and 8 + 16 will be checked)
                (16, TRANSPARENT)
            };

            self.state.line[ux] = if fc & SEMITRANS != 0 {
                blend_semitrans(effect, bldcnt, bldalpha, bldy, first, fc, second, sc)
            } else if bit(en_mask, 5) == 1 && effect != 0 {
                blend(effect, bldcnt, bldalpha, bldy, first, fc, second, sc)
            } else {
                fc
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alpha_blend1() {
        let c1 = colour_repack((31, 31, 0)) as u32;
        let c2 = colour_repack((0, 27, 31)) as u32;

        let bldalpha = (8 << 8) | (8 << 0);

        let cr = colour_unpack(alpha_blend(bldalpha, c1, c2) as u16);

        assert_eq!((15, 29, 15), cr);
    }

    #[test]
    fn test_alpha_blend2() {
        let c1 = colour_repack((0, 0, 0)) as u32;
        let c2 = colour_repack((31, 31, 31)) as u32;

        let bldalpha = (15 << 8) | (16 << 0);

        let cr = colour_unpack(alpha_blend(bldalpha, c1, c2) as u16);

        assert_eq!((29, 29, 29), cr);
    }
}
