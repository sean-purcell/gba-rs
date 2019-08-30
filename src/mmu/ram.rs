use std::vec::Vec;

use super::{Mmu, MemoryRead, MemoryUnit, bytes};

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
    fn load8(&self, addr: u32) -> MemoryRead<u8> {
        bytes::load8(self.mem.as_slice(), addr)
    }

    fn set8(&mut self, addr: u32, val: u8) {
        bytes::set8(self.mem.as_mut_slice(), addr, val)
    }

    fn load16(&self, addr: u32) -> MemoryRead<u16> {
        bytes::load16(self.mem.as_slice(), addr)
    }

    fn set16(&mut self, addr: u32, val: u16) {
        bytes::set16(self.mem.as_mut_slice(), addr, val)
    }

    fn load32(&self, addr: u32) -> MemoryRead<u32> {
        bytes::load32(self.mem.as_slice(), addr)
    }

    fn set32(&mut self, addr: u32, val: u32) {
        bytes::set32(self.mem.as_mut_slice(), addr, val)
    }
}

// FIXME bad hack because Mmu and MemoryUnit are conflicting traits
pub struct RamUnit {
    pub ram: Ram,
}

impl MemoryUnit for RamUnit {
    fn load8(&self, addr: u32) -> u8 {
        use self::MemoryRead::*;

        match self.ram.load8(addr) {
            Value(x) => x,
            Open => panic!("Accessing out of bounds memory")
        }
    }

    fn set8(&mut self, addr: u32, val: u8) {
        self.ram.set8(addr, val);
    }

    fn load16(&self, addr: u32) -> u16 {
        use self::MemoryRead::*;

        match self.ram.load16(addr) {
            Value(x) => x,
            Open => panic!("Accessing out of bounds memory")
        }
    }

    fn set16(&mut self, addr: u32, val: u16) {
        self.ram.set16(addr, val);
    }

    fn load32(&self, addr: u32) -> u32 {
        use self::MemoryRead::*;

        match self.ram.load32(addr) {
            Value(x) => x,
            Open => panic!("Accessing out of bounds memory")
        }
    }

    fn set32(&mut self, addr: u32, val: u32) {
        self.ram.set32(addr, val);
    }
}
