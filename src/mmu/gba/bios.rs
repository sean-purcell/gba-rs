use cpu::Cpu;
use rom::GameRom;
use shared::Shared;

use super::{Gba, Mmu, MemoryRead};

const BIOS_SIZE: u32 = 0x4000;

#[derive(Default)]
pub struct Bios<'a> {
    bios: GameRom,
    cpu: Shared<Cpu<Gba<'a>>>,
}

impl<'a> Bios<'a> {
    pub fn new(bios: GameRom) -> Self {
        Self {
            bios: bios,
            cpu: Default::default(),
        }
    }

    pub fn init(&mut self, cpu: Shared<Cpu<Gba<'a>>>) {
        self.cpu = cpu;
    }
}

impl<'a> Mmu for Bios<'a> {
    fn load8(&self, addr: u32) -> MemoryRead<u8> {
        // Determine where CPU PC is
        if addr < BIOS_SIZE {
            if self.cpu.get_prefetch_addr() < BIOS_SIZE {
                self.bios.load8(addr)
            } else {
                // Not allowed to read from BIOS memory
                // FIXME: add last-fetched bios instruction
                MemoryRead::Value(0)
            }
        } else {
            MemoryRead::Open
        }
    }

    fn set8(&mut self, addr: u32, val: u8) {
        self.bios.set8(addr, val)
    }

    fn load16(&self, addr: u32) -> MemoryRead<u16> {
        // Determine where CPU PC is
        if addr < BIOS_SIZE {
            if self.cpu.get_prefetch_addr() < BIOS_SIZE {
                self.bios.load16(addr)
            } else {
                // Not allowed to read from BIOS memory
                // FIXME: add last-fetched bios instruction
                MemoryRead::Value(0)
            }
        } else {
            MemoryRead::Open
        }
    }

    fn set16(&mut self, addr: u32, val: u16) {
        self.bios.set16(addr, val)
    }

    fn load32(&self, addr: u32) -> MemoryRead<u32> {
        // Determine where CPU PC is
        if addr < BIOS_SIZE {
            if self.cpu.get_prefetch_addr() < BIOS_SIZE {
                self.bios.load32(addr)
            } else {
                // Not allowed to read from BIOS memory
                // FIXME: add last-fetched bios instruction
                MemoryRead::Value(0)
            }
        } else {
            MemoryRead::Open
        }
    }

    fn set32(&mut self, addr: u32, val: u32) {
        self.bios.set32(addr, val)
    }
}
