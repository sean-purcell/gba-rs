pub mod bytes;
pub mod gba;
pub mod ram;

// Add result type for memory accesses here
#[derive(Debug)]
pub enum MemoryRead<T> {
    Value(T),
    Open,
}

impl<T> MemoryRead<T> {
    pub fn get(self) -> T {
        use self::MemoryRead::*;

        match self {
            Value(x) => x,
            Open => panic!("Attempted to force read of open address"),
        }
    }
}

/// A memory management unit for GBA, to handle memory accesses
pub trait MemoryUnit {
    fn load8(&self, addr: u32) -> u8;
    fn set8(&mut self, addr: u32, val: u8);
    fn load16(&self, addr: u32) -> u16;
    fn set16(&mut self, addr: u32, val: u16);
    fn load32(&self, addr: u32) -> u32;
    fn set32(&mut self, addr: u32, val: u32);
}

/// A subpiece of the MMU TODO: rename
pub trait Mmu {
    fn load8(&self, addr: u32) -> MemoryRead<u8>;

    fn set8(&mut self, addr: u32, val: u8);

    fn load16(&self, addr: u32) -> MemoryRead<u16>;

    fn set16(&mut self, addr: u32, val: u16);

    fn load32(&self, addr: u32) -> MemoryRead<u32>;

    fn set32(&mut self, addr: u32, val: u32);
}
