use std::vec::Vec;

use byteorder::{ByteOrder, LittleEndian};

use super::Mmu;

/// Implements a basic memory model with no memory mapping
pub struct Raw {
    mem: Vec<u8>,
}

impl Raw {
    pub fn new(size: usize) -> Raw {
        Raw { mem: vec![0u8; size] }
    }

    pub fn new_with_data(size: usize, data: &[u8]) -> Raw {
        let mut raw = Raw::new(size);
        raw.mem[..data.len()].clone_from_slice(data);
        raw
    }
}

impl Mmu for Raw {
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        let idx = addr as usize;
        self.mem[idx]
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        let idx = addr as usize;
        self.mem[idx] = val;
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::read_u32(&self.mem[idx..idx + 4])
    }

    #[inline]
    fn set32(&mut self, addr: u32, val: u32) {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::write_u32(&mut self.mem[idx..idx + 4], val)
    }
}
