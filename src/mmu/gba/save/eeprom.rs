use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
use serde::ser::SerializeTuple;

use shared::Shared;

use io::IoReg;

use mmu::Mmu;

const MEM_SIZE: usize = 1024;

pub struct Eeprom<'a> {
    ee: RefCell<EepromInner<'a>>,
}

struct EepromInner<'a> {
    mem: EepromMem,
    state: State,
    write: bool,
    addr: u16,
    bits: u8,
    data: u64,

    io: Shared<IoReg<'a>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
            mem: Default::default(),
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
                self.mem[self.addr as usize] = self.data;
                // Idle will return a 1 for a read, so it will "Confirm" the write
                self.reset();
            }
            ConfirmRead => {
                self.data = self.mem[self.addr as usize];
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

struct EepromMem([u64; MEM_SIZE]);

impl Default for EepromMem {
    fn default() -> Self {
        EepromMem([0u64; MEM_SIZE])
    }
}

impl Deref for EepromMem {
    type Target = [u64];
    fn deref(&self) -> &[u64] {
        &self.0
    }
}

impl DerefMut for EepromMem {
    fn deref_mut(&mut self) -> &mut [u64] {
        &mut self.0
    }
}

impl Serialize for EepromMem {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_tuple(MEM_SIZE)?;
        for val in self.0.iter() {
            s.serialize_element(val)?;
        }
        s.end()
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

impl<'a> Serialize for Eeprom<'a> {
    // Note: Everything other than mem shouldn't actually need serializing
    // DMA currently is instantaneous and so it can't span a frame barrier
    // (where the save state takes place)
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("gba_rs::gba::save::eeprom::Eeprom", 6)?;
        let ee = self.ee.borrow();

        s.serialize_field("mem", &ee.mem)?;
        s.serialize_field("state", &ee.state)?;
        s.serialize_field("write", &ee.write)?;
        s.serialize_field("addr", &ee.addr)?;
        s.serialize_field("bits", &ee.bits)?;
        s.serialize_field("data", &ee.data)?;
        s.end()
    }
}
