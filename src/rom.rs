use std::fmt;
use std::fs::File;
use std::path::Path;
use std::ops::Deref;

use memmap::{Mmap, MmapMut};

use mmu::{Mmu, MemoryRead, bytes};

use GBAError;
use Result;

pub struct GameRom {
    rom: Mmap,
}

impl GameRom {
    pub fn new(path: &Path) -> Result<GameRom> {
        match File::open(path) {
            Ok(file) => {
                match unsafe { Mmap::map(&file) } {
                    Ok(mmap) => Ok(GameRom { rom: mmap }),
                    Err(err) => Err(GBAError::RomLoadError(err)),
                }
            }
            Err(err) => Err(GBAError::RomLoadError(err)),
        }
    }
}

impl Default for GameRom {
    fn default() -> Self {
        return GameRom { rom:
            MmapMut::map_anon(0).unwrap()
                .make_read_only().unwrap() }
    }
}

impl Deref for GameRom {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.rom.deref()
    }
}

fn warning<T>(addr: u32, val: T)
where
    T: fmt::LowerHex,
{
    warn!(
        "Attempted to store to game ROM: a: {:#010x}, v: {:#x}",
        addr,
        val
    );
}

impl Mmu for GameRom {
    fn load8(&self, addr: u32) -> MemoryRead<u8> {
        if (addr as usize) < self.rom.len() {
            bytes::load8(self.deref(), addr)
        } else {
            MemoryRead::Value(0)
        }
    }

    fn set8(&mut self, addr: u32, val: u8) {
        warning(addr, val);
    }

    fn load16(&self, addr: u32) -> MemoryRead<u16> {
        if (addr as usize) < self.rom.len() {
            bytes::load16(self.deref(), addr)
        } else {
            MemoryRead::Value(0)
        }
    }

    fn set16(&mut self, addr: u32, val: u16) {
        warning(addr, val);
    }

    fn load32(&self, addr: u32) -> MemoryRead<u32> {
        if (addr as usize) < self.rom.len() {
            bytes::load32(self.deref(), addr)
        } else {
            MemoryRead::Value(0)
        }
    }

    fn set32(&mut self, addr: u32, val: u32) {
        warning(addr, val);
    }
}

impl fmt::Debug for GameRom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("GameRom")
            .field("len", &self.rom.len())
            .field("ptr", &self.rom.as_ptr())
            .field("val", &format!("{:#x}", self.rom[0xb2]))
            .finish()
    }
}
