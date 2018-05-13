use std::cell::RefCell;

use byteorder::{ByteOrder, LittleEndian};
use shared::Shared;

use io::IoReg;

use mmu::Mmu;

const MEM_SIZE: usize = 8192;

pub struct Eeprom<'a> {
    ee: RefCell<EepromInner<'a>>,
}

struct EepromInner<'a> {
    mem: [u8; MEM_SIZE],
    state: State,
    write: bool,
    addr: u16,
    bits: u8,
    data: u64,

    io: Shared<IoReg<'a>>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Idle,
    ReadAddress,
    WriteData,
    ConfirmWrite,
    ConfirmRead,
    ReadData,
}

impl<'a> Default for Eeprom<'a> {
    fn default() -> Self {
        Eeprom { ee: RefCell::new(Default::default()) }
    }
}

impl<'a> Eeprom<'a> {
    pub fn init(&mut self, io: Shared<IoReg<'a>>) {
        self.ee.borrow_mut().init(io);
    }
}

impl<'a> Default for EepromInner<'a> {
    fn default() -> Self {
        EepromInner {
            mem: [0u8; MEM_SIZE],
            state: State::Idle,
            write: false,
            addr: 0,
            bits: 0,
            data: 0,
            io: Shared::empty(),
        }
    }
}

impl<'a> EepromInner<'a> {
    pub fn init(&mut self, io: Shared<IoReg<'a>>) {
        self.io = io;
    }

    fn reset(&mut self) {
        self.state = State::Idle;
        self.write = false;
        self.addr = 0;
        self.bits = 0;
        self.data = 0;
    }

    pub fn write(&mut self, val: u16) {
        use self::State::*;

        let dma_len = self.io.dma_length();
        if dma_len == 0 {
            return;
        }

        let bit = val & 1;
        debug!("eeprom write: {}, current state: {:?}", bit, self.state);

        match self.state {
            Idle => {
                if bit == 1 {
                    self.state = ReadAddress;
                    self.bits = 1;
                }
            }
            ReadAddress => {
                if self.bits < 2 {
                    self.write = bit == 0;
                    self.bits = 2;
                } else {
                    self.addr = (self.addr << 1) | bit;
                    self.bits += 1;
                    let done = if self.bits == 16 && (dma_len == 17 || dma_len == 81) {
                        // assume 14-bit bus width
                        // FIXME: might be worth checking if the 0 is valid
                        self.addr &= 0x3ff;
                        true
                    } else if self.bits == 8 && (dma_len == 9 || dma_len == 73) {
                        true
                    } else {
                        false
                    };
                    if done {
                        self.bits = 0;
                        self.state = if self.write { WriteData } else { ConfirmRead };
                    }
                }
            }
            WriteData => {
                self.data = (self.data << 1) | (bit as u64);
                self.bits += 1;
                if self.bits == 64 {
                    self.state = ConfirmWrite;
                }
            }
            ConfirmWrite => {
                let addr = self.addr as usize * 8;
                LittleEndian::write_u64(&mut self.mem[addr..addr + 8], self.data);
                // Idle will return a 1 for a read, so it will "Confirm" the write
                self.reset();
            }
            ConfirmRead => {
                let addr = self.addr as usize * 8;
                self.data = LittleEndian::read_u64(&mut self.mem[addr..addr + 8]);
                self.bits = 0;
                self.state = ReadData;
            }
            ReadData => {
                // no-op, user should be reading
            }
        }
    }

    pub fn read(&mut self) -> u16 {
        use self::State::*;
        let istate = self.state;
        let bit = match self.state {
            Idle | ReadAddress | WriteData | ConfirmWrite | ConfirmRead => 1,
            ReadData => {
                let res = if self.bits < 3 {
                    0
                } else {
                    let bit = 67 - self.bits;
                    ((self.data >> bit) & 1) as u16
                };
                self.bits += 1;
                if self.bits == 68 {
                    self.reset();
                }
                res
            }
        };
        debug!("eeprom read: {}, current state: {:?}", bit, istate);
        bit
    }
}

impl<'a> Mmu for Eeprom<'a> {
    #[inline]
    fn load8(&self, _addr: u32) -> u8 {
        self.ee.borrow_mut().read() as u8
    }

    #[inline]
    fn set8(&mut self, _addr: u32, val: u8) {
        self.ee.borrow_mut().write(val as u16)
    }

    #[inline]
    fn load16(&self, _addr: u32) -> u16 {
        self.ee.borrow_mut().read()
    }

    #[inline]
    fn set16(&mut self, _addr: u32, val: u16) {
        self.ee.borrow_mut().write(val)
    }

    #[inline]
    fn load32(&self, _addr: u32) -> u32 {
        self.ee.borrow_mut().read() as u32
    }

    #[inline]
    fn set32(&mut self, _addr: u32, val: u32) {
        self.ee.borrow_mut().write(val as u16)
    }
}
