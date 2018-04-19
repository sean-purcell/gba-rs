use std::fmt;
use std::fs::File;
use std::path::Path;
use std::ops::Deref;

use memmap::Mmap;

use mmu::Mmu;

use GBAError;
use Result;

pub struct GameRom {
    file: File,
    rom: Mmap,
}

impl GameRom {
    pub fn new(path: &Path) -> Result<GameRom> {
        match File::open(path) {
            Ok(file) => {
                match unsafe { Mmap::map(&file) } {
                    Ok(mmap) => Ok(GameRom {
                        file: file,
                        rom: mmap,
                    }),
                    Err(err) => Err(GBAError::RomLoadError(err)),
                }
            }
            Err(err) => Err(GBAError::RomLoadError(err)),
        }
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
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        self.deref().load8(addr)
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        warning(addr, val);
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        self.deref().load16(addr)
    }

    #[inline]
    fn set16(&mut self, addr: u32, val: u16) {
        warning(addr, val);
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        self.deref().load32(addr)
    }

    #[inline]
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
