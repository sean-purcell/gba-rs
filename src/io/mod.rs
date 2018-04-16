pub mod ppu;

use mmu::ram::Ram;

const IO_REG_SIZE: usize = 0x804;

struct IoReg {
    reg: Ram,
}

impl IoReg {
    pub fn new() -> IoReg {
        let mut reg = IoReg { reg: Ram::new(IO_REG_SIZE) };

        reg.init();

        reg
    }

    fn init(&mut self) {
        unimplemented!();
    }
}
