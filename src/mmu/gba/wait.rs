#[derive(Serialize, Deserialize, Default)]
pub struct WaitStates {
    rom: [[u8; 2]; 3],
    sram: u8,
}

impl WaitStates {
    pub fn new() -> Self {
        let mut w = Default::default();
        w.set_waitcnt(0);
        w
    }

    pub fn set_waitcnt(&mut self, w: u16) {
        use bit_util::{bit, extract};

        self.sram = [4, 3, 2, 8][extract(w, 0, 2) as usize];
        self.rom[0][0] = [4, 3, 2, 8][extract(w, 2, 2) as usize];
        self.rom[0][1] = [2, 1][bit(w, 4) as usize];
        self.rom[1][0] = [4, 3, 2, 8][extract(w, 2, 2) as usize];
        self.rom[1][1] = [4, 1][bit(w, 4) as usize];
        self.rom[2][0] = [4, 3, 2, 8][extract(w, 2, 2) as usize];
        self.rom[2][1] = [8, 1][bit(w, 4) as usize];
    }

    fn get_waits_range(&self, range: u8, seq: bool) {
        let i = if seq { 1 } else { 0 };
        match range {
            0x8 | 0x9 => self.rom[0][i],
            0xA | 0xB => self.rom[1][i],
            0xC | 0xD => self.rom[2][i],
            0xE => self.sram,
        } + 1
    }

    pub fn get_waitstates(&self, addr: u32, width: u8, seq: bool) {
        use bit_util::extract;
        let range = extract(addr, 24, 4);
        self.get_waits_range(range, seq) +
            if width == 4 { self.get_waits_range(range, true) }
            else { 0 }
    }
}
