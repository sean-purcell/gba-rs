use rom::GameRom;

use super::Mmu;
use super::ram::Ram;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum MemoryRange {
    Bios,
    BoardWram,
    ChipWram,
    GamePakRom,
    GamePakSram,
    Unused,
}

const RANGES: [MemoryRange; 5] = [
    MemoryRange::Bios,
    MemoryRange::BoardWram,
    MemoryRange::ChipWram,
    MemoryRange::GamePakRom,
    MemoryRange::GamePakSram,
];

impl MemoryRange {
    fn bounds(&self) -> (u32, u32) {
        use self::MemoryRange::*;
        #[cfg_attr(rustfmt, rustfmt_skip)]
        match *self {
            Bios        => (0x00000000, 0x00004000),
            BoardWram   => (0x02000000, 0x02040000),
            ChipWram    => (0x03000000, 0x03008000),
            GamePakRom  => (0x08000000, 0x0E000000),
            GamePakSram => (0x0E000000, 0x0E010000),
            Unused      => (0x00000000, 0xFFFFFFFF),
        }
    }

    pub fn convert_addr(&self, addr: u32) -> u32 {
        use self::MemoryRange::*;
        match *self {
            GamePakRom => addr & 0x1FFFFFF,
            _ => addr - self.bounds().0,
        }
    }

    pub fn match_addr(addr: u32) -> MemoryRange {
        for range in RANGES.iter() {
            let (l, u) = range.bounds();
            if l <= addr && addr < u {
                return *range;
            }
        }
        MemoryRange::Unused
    }
}

/// Implements the memory mapping for a GBA system
pub struct Gba {
    bram: Ram,
    cram: Ram,
    rom: GameRom,
    gram: Ram,
}

impl Gba {
    pub fn new_with_rom(rom: GameRom) -> Gba {
        Gba {
            bram: Ram::new(256 * 1024),
            cram: Ram::new(32 * 1024),
            rom: rom,
            gram: Ram::new(64 * 1024),
        }
    }

    pub fn get_range(&self, addr: u32) -> Option<(u32, &Mmu)> {
        use self::MemoryRange::*;
        let range = MemoryRange::match_addr(addr);
        let naddr = range.convert_addr(addr);
        match range {
            BoardWram => Some((naddr, &self.bram)),
            ChipWram => Some((naddr, &self.cram)),
            GamePakRom => Some((naddr, &self.rom)),
            GamePakSram => Some((naddr, &self.gram)),
            _ => None,
        }
    }

    pub fn get_range_mut(&mut self, addr: u32) -> Option<(u32, &mut Mmu)> {
        use self::MemoryRange::*;
        let range = MemoryRange::match_addr(addr);
        let naddr = range.convert_addr(addr);
        match range {
            BoardWram => Some((naddr, &mut self.bram)),
            ChipWram => Some((naddr, &mut self.cram)),
            GamePakRom => Some((naddr, &mut self.rom)),
            GamePakSram => Some((naddr, &mut self.gram)),
            _ => None,
        }
    }
}

fn warning(addr: u32) {
    warn!("Access to unmapped memory: {:#010x}", addr);
}

impl Mmu for Gba {
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        match self.get_range(addr) {
            Some((naddr, mmu)) => mmu.load8(naddr),
            None => {
                warning(addr);
                0
            }
        }
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        match self.get_range_mut(addr) {
            Some((naddr, mmu)) => mmu.set8(naddr, val),
            None => warning(addr),
        }
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        match self.get_range(addr) {
            Some((naddr, mmu)) => mmu.load16(naddr),
            None => {
                warning(addr);
                0
            }
        }
    }

    #[inline]
    fn set16(&mut self, addr: u32, val: u16) {
        match self.get_range_mut(addr) {
            Some((naddr, mmu)) => mmu.set16(naddr, val),
            None => warning(addr),
        }
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        match self.get_range(addr) {
            Some((naddr, mmu)) => mmu.load32(naddr),
            None => {
                warning(addr);
                0
            }
        }
    }

    #[inline]
    fn set32(&mut self, addr: u32, val: u32) {
        match self.get_range_mut(addr) {
            Some((naddr, mmu)) => mmu.set32(naddr, val),
            None => warning(addr),
        }
    }
}
