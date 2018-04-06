use std::fmt;
use std::fs::File;
use std::path::Path;
use std::ops::Deref;

use memmap::Mmap;

use GBAError;
use Result;

pub struct Rom {
    file: File,
    rom: Mmap,
}

impl Rom {
    pub fn new(path: &Path) -> Result<Rom> {
        match File::open(path) {
            Ok(file) => {
                match unsafe { Mmap::map(&file) } {
                    Ok(mmap) => Ok(Rom {
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

impl Deref for Rom {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.rom.deref()
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::os::unix::io::AsRawFd;
        fmt.debug_struct("Rom")
            .field("fd", &self.file.as_raw_fd())
            .field("len", &self.rom.len())
            .field("ptr", &self.rom.as_ptr())
            .field("val", &format!("{:#x}", self.rom[0xb2]))
            .finish()
    }
}
