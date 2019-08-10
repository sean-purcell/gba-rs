use byteorder::{ByteOrder, LittleEndian};

pub mod gba;
pub mod ram;

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

impl<'a> Mmu for &'a mut [u8] {
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        let idx = addr as usize;
        self[idx]
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        let idx = addr as usize;
        self[idx] = val;
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        debug_assert!(addr % 2 == 0);
        let idx = addr as usize;
        LittleEndian::read_u16(&self[idx..idx + 2])
    }

    #[inline]
    fn set16(&mut self, addr: u32, val: u16) {
        debug_assert!(addr % 2 == 0);
        let idx = addr as usize;
        LittleEndian::write_u16(&mut self[idx..idx + 2], val)
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::read_u32(&self[idx..idx + 4])
    }

    #[inline]
    fn set32(&mut self, addr: u32, val: u32) {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::write_u32(&mut self[idx..idx + 4], val)
    }
}

impl<'a> Mmu for &'a [u8] {
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        let idx = addr as usize;
        self[idx]
    }

    #[inline]
    fn set8(&mut self, _addr: u32, _val: u8) {
        unreachable!()
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        debug_assert!(addr % 2 == 0);
        let idx = addr as usize;
        if (idx >= self.len()) {
            println!("Hit badness");
        }
        LittleEndian::read_u16(&self[idx..idx + 2])
    }

    #[inline]
    fn set16(&mut self, _addr: u32, _val: u16) {
        unreachable!()
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        debug_assert!(addr % 4 == 0);
        let idx = addr as usize;
        LittleEndian::read_u32(&self[idx..idx + 4])
    }

    #[inline]
    fn set32(&mut self, _addr: u32, _val: u32) {
        unreachable!()
    }
}
