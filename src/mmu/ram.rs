use std::vec::Vec;

use super::Mmu;

/// Implements a basic memory model with no memory mapping
#[derive(Serialize, Deserialize)]
pub struct Ram {
    mem: Vec<u8>,
}

impl Ram {
    pub fn new(size: usize) -> Ram {
        Ram { mem: vec![0u8; size] }
    }

    pub fn new_with_data(size: usize, data: &[u8]) -> Ram {
        let mut ram = Ram::new(size);
        ram.mem[..data.len()].clone_from_slice(data);
        ram
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }
}

impl Mmu for Ram {
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        self.mem.as_slice().load8(addr)
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        self.mem.as_mut_slice().set8(addr, val)
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        self.mem.as_slice().load16(addr)
    }

    #[inline]
    fn set16(&mut self, addr: u32, val: u16) {
        self.mem.as_mut_slice().set16(addr, val)
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        self.mem.as_slice().load32(addr)
    }

    #[inline]
    fn set32(&mut self, addr: u32, val: u32) {
        self.mem.as_mut_slice().set32(addr, val)
    }
}
