pub mod gba;
pub mod raw;

/// A memory management unit for GBA, to handle memory accesses
pub trait Mmu {
    #[inline]
    fn load8(&self, addr: u32) -> u8;

    #[inline]
    fn set8(&mut self, addr: u32, val: u8);

    #[inline]
    fn load16(&self, addr: u32) -> u16;

    #[inline]
    fn set16(&mut self, addr: u32, val: u16);

    #[inline]
    fn load32(&self, addr: u32) -> u32;

    #[inline]
    fn set32(&mut self, addr: u32, val: u32);
}
