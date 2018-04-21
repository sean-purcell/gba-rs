use super::*;
use super::rotscale::*;

impl<'a> Ppu<'a> {
    pub(super) fn render_line_mode3(&mut self, row: u32, dspcnt: u16) {
        let in_win0 = (dspcnt & (1 << 13)) != 0 && in_win_vert(self.io.get_priv(0x44), row);
        let in_win1 = (dspcnt & (1 << 14)) != 0 && in_win_vert(self.io.get_priv(0x46), row);

        for x in 0..COLS {
            let idx = row * COLS + x;
            self.state.line[x as usize] = self.mmu.vram.load16(idx * 2) as u32;
        }

        let rparams = RotScaleParams::new(
            self.io.get_priv(0x20),
            self.io.get_priv(0x22),
            self.io.get_priv(0x24),
            self.io.get_priv(0x26),
        );

        render_rotscale_line(
            &mut self.state.line,
            row,
            &self.mmu,
            &mut self.state.bg2ref,
            rparams,
            BgControl::Bitmap(dspcnt),
        );
    }
}
