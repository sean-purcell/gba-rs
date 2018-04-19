pub mod ppu;

use mmu::Mmu;
use mmu::ram::Ram;

const IO_REG_SIZE: usize = 0x804;

pub struct IoReg {
    reg: Ram,
}

impl IoReg {
    pub fn new() -> IoReg {
        let mut reg = IoReg { reg: Ram::new(IO_REG_SIZE) };

        reg
    }

    fn init(&mut self) {
        unimplemented!();
    }

    fn get_priv(&self, addr: u32) -> u16 {
        self.reg.load16(addr)
    }

    fn set(&self, addr: u32, val: u16) {}
}

// TODO: implement read/write masks
impl Mmu for IoReg {
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        self.reg.load8(addr)
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        self.reg.set8(addr, val)
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        self.reg.load16(addr)
    }

    #[inline]
    fn set16(&mut self, addr: u32, val: u16) {
        self.reg.set16(addr, val)
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        self.reg.load32(addr)
    }

    #[inline]
    fn set32(&mut self, addr: u32, val: u32) {
        self.reg.set32(addr, val)
    }
}
