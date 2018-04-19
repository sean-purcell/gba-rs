pub mod ppu;

use self::ppu::Ppu;

use mmu::Mmu;
use mmu::ram::Ram;
use shared::Shared;

const IO_REG_SIZE: usize = 0x804;

pub struct IoReg<'a> {
    reg: Ram,

    ppu: Shared<Ppu<'a>>,
}

impl<'a> IoReg<'a> {
    pub fn new() -> Self {
        IoReg {
            reg: Ram::new(IO_REG_SIZE),
            ppu: Shared::empty(),
        }
    }

    pub fn init(&mut self, ppu: Shared<Ppu<'a>>) {
        self.ppu = ppu;
    }

    #[inline]
    fn get_priv(&self, addr: u32) -> u16 {
        self.reg.load16(addr)
    }

    #[inline]
    fn set_priv(&mut self, addr: u32, val: u16) {
        self.reg.set16(addr, val);
    }

    #[inline]
    fn set(&mut self, addr: u32, val: u16) {
        let ro = ro_mask(addr);
        let old = if ro != 0 {
            self.get_priv(addr)
        } else {
            0
        };
        let nval = (ro & old) | (!ro & val);
        self.reg.set16(addr, nval);

        // Potential callback
        self.updated(addr, nval);
    }

    fn updated(&mut self, addr: u32, val: u16) {
        match addr {
            0x28 | 0x2a | 0x2c | 0x2e => self.ppu.update_bg2ref(),
            0x38 | 0x3a | 0x3c | 0x3e => self.ppu.update_bg3ref(),
            _ => (),
        }
    }
}

// TODO: implement read/write masks
impl<'a> Mmu for IoReg<'a> {
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

/// Mapping from address to a mask representing the write-protected parts
/// of the field
fn ro_mask(addr: u32) -> u16 {
    use std::u16;
    let all = u16::MAX;

    match addr {
        _ => 0,
    }
}
