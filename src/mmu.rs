use byteorder::{ByteOrder, LittleEndian};

const MEM_SIZE: usize = 0x10000;

/// A memory management unit for GBA, to handle memory accesses
pub struct Mmu {
    mem: [u8; MEM_SIZE],
}

impl Mmu {
    pub fn new() -> Mmu {
        Mmu { mem: [0u8; MEM_SIZE] }
    }

    #[inline]
    pub fn load8(&self, addr: u32) -> u8 {
        let idx = addr as usize;
        self.mem[idx]
    }

    #[inline]
    pub fn set8(&mut self, addr: u32, val: u8) {
        let idx = addr as usize;
        self.mem[idx] = val;
    }

    #[inline]
    pub fn load32(&self, addr: u32) -> u32 {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::read_u32(&self.mem[idx..idx + 4])
    }

    #[inline]
    pub fn set32(&mut self, addr: u32, val: u32) {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::write_u32(&mut self.mem[idx..idx + 4], val)
    }
}
