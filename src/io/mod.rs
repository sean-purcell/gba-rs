mod dma;
pub mod key;
pub mod ppu;
pub mod spu;
mod timer;

use self::dma::Dma;
use self::ppu::Ppu;
use self::timer::Timers;

use cpu::{exception, Cpu};
use mmu::gba::Gba as GbaMmu;
use mmu::ram::Ram;
use mmu::{MemoryRead, Mmu};
use shared::Shared;

const IO_REG_SIZE: usize = 0x804;

const DSPCNT: u32 = 0x0;
const DISPSTAT: u32 = 0x4;
const VCOUNT: u32 = 0x6;
const KEYINPUT: u32 = 0x130;
const KEYCNT: u32 = 0x132;
const IE: u32 = 0x200;
const IF: u32 = 0x202;
const IME: u32 = 0x208;

#[derive(Serialize, Deserialize)]
pub struct IoReg<'a> {
    reg: Ram,

    #[serde(skip)]
    cpu: Shared<Cpu<GbaMmu<'a>>>,
    #[serde(skip)]
    mmu: Shared<GbaMmu<'a>>,
    #[serde(skip)]
    ppu: Shared<Ppu<'a>>,

    timers: Timers<'a>,
    dma: Dma<'a>,
}

impl<'a> IoReg<'a> {
    pub fn new() -> Self {
        let mut io = IoReg {
            reg: Ram::new(IO_REG_SIZE),
            cpu: Shared::empty(),
            mmu: Shared::empty(),
            ppu: Shared::empty(),
            timers: Default::default(),
            dma: Default::default(),
        };
        io.set_initial();
        io
    }

    fn set_initial(&mut self) {
        self.reg.set16(0x20, 0x100);
        self.reg.set16(0x26, 0x100);
        self.reg.set16(0x30, 0x100);
        self.reg.set16(0x36, 0x100);
    }

    pub fn init(
        &mut self,
        cpu: Shared<Cpu<GbaMmu<'a>>>,
        mmu: Shared<GbaMmu<'a>>,
        ppu: Shared<Ppu<'a>>,
    ) {
        self.cpu = cpu;
        self.mmu = mmu;
        self.ppu = ppu;

        let io = Shared::new(self);
        self.timers.init(io);
        self.dma.init(io);
    }

    pub fn cycle(&mut self) {
        self.timers.cycle();
        self.check_interrupt();
    }

    pub fn dma_length(&self) -> u32 {
        self.dma.length()
    }

    fn check_interrupt(&mut self) {
        let ir = self.get_priv(IF); // IF register, if is a keyword though
        if (self.get_priv(IME) & 1) != 0 && ir != 0 && self.cpu.irq_enable() {
            let ie = self.get_priv(IE);
            if ir & ie != 0 {
                self.cpu.exception(&exception::Exception::Interrupt);
            }
        }
    }

    fn raise_interrupt(&mut self, itr: u8) {
        let pif = self.get_priv(IF);
        self.set_priv(IF, pif | (1 << (itr as u16)));

        info!("Interrupt {} raised", itr);
    }

    fn get_priv(&self, addr: u32) -> u16 {
        self.reg.load16(addr).get()
    }

    fn set_priv(&mut self, addr: u32, val: u16) {
        self.reg.set16(addr, val);
    }

    fn get(&self, addr: u32) -> MemoryRead<u16> {
        use self::MemoryRead::*;

        if !readable(addr) {
            // If the other half of the 32 bit range is readable, t his is just
            // 0 instead of open bus.
            return if readable(addr ^ 2) { Value(0) } else { Open };
        }
        let val = match addr {
            0x100 | 0x104 | 0x108 | 0x10c => self.timers.get((addr - 0x100) / 4),
            _ => self.reg.load16(addr).get(),
        };
        let wo = wo_mask(addr);
        Value(val & !wo)
    }

    fn set(&mut self, addr: u32, val: u16) {
        if !writable(addr) {
            warn!(
                "Writing to non-writable IO register: {:#010x} -> {:#06x}",
                addr, val
            );
            // If not writable, no point in doing anything
            return;
        }
        let ro = ro_mask(addr);
        let old = self.get_priv(addr);
        let nval = (ro & old) | (!ro & val);
        self.reg.set16(addr, nval);

        // Potential callback
        self.updated(addr, old, nval);
    }

    fn updated(&mut self, addr: u32, old: u16, new: u16) {
        match addr {
            0x28 | 0x2a | 0x2c | 0x2e => self.ppu.update_bg2ref(),
            0x38 | 0x3a | 0x3c | 0x3e => self.ppu.update_bg3ref(),
            0xBA | 0xC6 | 0xD2 | 0xDE => self.dma.updated(addr - 0xB0, old, new),
            0x102 | 0x106 | 0x10a | 0x10e => self.timers.updated((addr - 0x102) / 4, old, new),
            0x130 => {
                let keycnt = self.get_priv(KEYCNT);
                self.check_key_intr(keycnt, new);
            }
            0x132 => {
                let keyinput = self.get_priv(KEYINPUT);
                self.check_key_intr(keyinput, new);
            }
            0x202 => self.disable_intrreq(new),
            _ => (),
        }
    }

    fn disable_intrreq(&mut self, val: u16) {
        let nv = self.get_priv(IF) & !val;
        self.set_priv(IF, nv);
    }
}

// TODO: implement read/write masks
impl<'a> Mmu for IoReg<'a> {
    fn load8(&self, addr: u32) -> MemoryRead<u8> {
        use self::MemoryRead::*;

        match self.get(addr & !1) {
            Value(x) => Value((x >> ((addr & 1) * 8)) as u8),
            Open => Open,
        }
    }

    fn set8(&mut self, addr: u32, val: u8) {
        let pv = if (addr as usize) < self.reg.len() {
            self.get_priv(addr & !1)
        } else {
            0
        };
        let shift = (addr & 1) * 8;
        let mask = 0xff00 >> shift;
        let nv = (pv & mask) | ((val as u16) << shift);
        self.set(addr & !1, nv);
    }

    fn load16(&self, addr: u32) -> MemoryRead<u16> {
        self.get(addr)
    }

    fn set16(&mut self, addr: u32, val: u16) {
        self.set(addr, val);
    }

    fn load32(&self, addr: u32) -> MemoryRead<u32> {
        use self::MemoryRead::*;

        // If one register is non-open, the other one will be as well
        match self.get(addr) {
            Value(x) => Value((x as u32) | ((self.get(addr + 2).get() as u32) << 16)),
            Open => Open,
        }
    }

    fn set32(&mut self, addr: u32, val: u32) {
        // This setting order is correct for timers, but there may be
        // simultaneous setting issues elsewhere, unaware of any so far.
        self.set(addr, val as u16);
        self.set(addr + 2, (val >> 16) as u16);
    }
}

// Enum would be cleaner, but this aligns better.
// 1 is readable, 2 is writable.
const READWRITABLE: [u8; 0x108] = [
    // LCD: 000
    3, 3, 3, 1, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 3, 3, 2, 0, 3, 3, 2, 0, 0, 0, 0, 0, // SOUND: 060
    3, 3, 3, 0, 3, 0, 3, 0, 3, 3, 3, 0, 3, 0, 3, 0, 3, 3, 3, 0, 3, 0, 0, 0, 3, 3, 3, 3, 3, 3, 3, 3,
    2, 2, 2, 2, 0, 0, 0, 0, // DMA: 0B0
    2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, // TIMER: 100
    3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, // SERIAL + KEYPAD: 120
    3, 3, 3, 3, 3, 3, 0, 0, 1, 3, 3, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3, 3, 3, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // SYSTEM CONTROL: 200
    3, 3, 3, 0, 3, 0, 0, 0,
];

fn writable(addr: u32) -> bool {
    debug_assert!(addr & 1 == 0);

    let idx = (addr / 2) as usize;
    if idx < 0x108 {
        (READWRITABLE[idx] & 2) != 0
    } else {
        match addr {
            0x300 | 0x800 | 0x802 => true,
            _ => false,
        }
    }
}

fn readable(addr: u32) -> bool {
    debug_assert!(addr & 1 == 0);

    let idx = (addr / 2) as usize;
    if idx < 0x108 {
        (READWRITABLE[idx] & 1) != 0
    } else {
        match addr {
            0x300 | 0x800 | 0x802 => true,
            _ => false,
        }
    }
}

/// Mapping from address to a mask representing the write-protected parts
/// of the field
fn ro_mask(addr: u32) -> u16 {
    use std::u16;

    match addr {
        0x004 => 0x0047,
        0x084 => 0x000f,
        _ => 0,
    }
}

fn wo_mask(addr: u32) -> u16 {
    use std::u16;

    match addr {
        0x062 => 0x001f,
        0x064 => 0x87ff,
        0x068 => 0x001f,
        0x06C => 0x87ff,
        0x072 => 0x00ff,
        0x074 => 0x87ff,
        0x078 => 0x001f,
        0x07C => 0x8000,
        0x300 => 0xff00,
        _ => 0,
    }
}
