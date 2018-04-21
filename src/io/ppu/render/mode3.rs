use std::cmp::min;

use super::*;
use super::rotscale::*;

impl<'a> Ppu<'a> {
    pub(super) fn render_line_mode3(&mut self, row: u32, dspcnt: u16) {
        let win_enable = extract(dspcnt as u32, 13, 3) != 0;
        let in_win0 = bit(dspcnt as u32, 13) == 1 && in_win_vert(self.io.get_priv(0x44), row);
        let in_win1 = bit(dspcnt as u32, 14) == 1 && in_win_vert(self.io.get_priv(0x46), row);

        // FIXME: bg2ref won't get updated if this is skipped, not sure
        // what the appropriate behaviour is
        let bg2en = bit(dspcnt as u32, 10) == 1;
        if bg2en {
            let rparams = RotScaleParams::new(
                self.io.get_priv(0x20),
                self.io.get_priv(0x22),
                self.io.get_priv(0x24),
                self.io.get_priv(0x26),
            );

            render_rotscale_line(
                &mut self.state.line2,
                row,
                &self.mmu,
                &mut self.state.bg2ref,
                rparams,
                BgControl::Bitmap(dspcnt),
            );
        }

        let winin = self.io.get_priv(0x48);
        let winout = self.io.get_priv(0x4a);

        let win0h = if in_win0 { self.io.get_priv(0x40) } else { 0 };
        let win1h = if in_win1 { self.io.get_priv(0x42) } else { 0 };

        let backdrop = (self.mmu.pram.load16(0) as u32) | (0x4 << 28);

        for x in 0..COLS {
            let ux = x as usize;
            let en_mask = if in_win0 && in_win_hori(win0h, x) {
                winin & 0xff
            } else if in_win1 && in_win_hori(win1h, x) {
                winin >> 8
            } else if false
            /* FIXME: obj window */
            {
                winout & 0xff
            } else if win_enable {
                winout >> 8
            } else {
                0xff
            } as u32;

            // FIXME: objects

            // FIXME: special blend effects
            let mut colour = backdrop;

            if bg2en && bit(en_mask, 2) == 1 {
                let nc = self.state.line2[ux];
                colour = min(colour, nc);
            }

            self.state.line[ux] = colour;
        }
    }
}
