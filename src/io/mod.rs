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
}
