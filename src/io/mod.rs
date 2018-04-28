pub mod ppu;
pub mod key;
mod timer;

use self::ppu::Ppu;
use self::timer::Timers;

use cpu::{Cpu, reg, exception};
use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;
use mmu::ram::Ram;
use shared::Shared;

const IO_REG_SIZE: usize = 0x804;

const DSPCNT: u32 = 0x0;
const DISPSTAT: u32 = 0x4;
const VCOUNT: u32 = 0x6;
const IE: u32 = 0x200;
const IF: u32 = 0x202;
const IME: u32 = 0x208;

pub struct IoReg<'a> {
    reg: Ram,

    cpu: Shared<Cpu<GbaMmu<'a>>>,
    ppu: Shared<Ppu<'a>>,

    timers: Timers<'a>,
}

impl<'a> IoReg<'a> {
    pub fn new() -> Self {
        let mut io = IoReg {
            reg: Ram::new(IO_REG_SIZE),
            cpu: Shared::empty(),
            ppu: Shared::empty(),
            timers: Default::default(),
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

    pub fn init(&mut self, cpu: Shared<Cpu<GbaMmu<'a>>>, ppu: Shared<Ppu<'a>>) {
        self.cpu = cpu;
        self.ppu = ppu;

        let io = Shared::new(self);
        self.timers.init(io);
    }

    pub fn cycle(&mut self) {
        self.timers.cycle();
        self.check_interrupt();
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

    #[inline]
    fn raise_interrupt(&mut self, itr: u8) {
        let pif = self.get_priv(IF);
        self.set_priv(IF, pif | (1 << (itr as u16)));

        info!("Interrupt {} raised", itr);
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
    fn get(&self, addr: u32) -> u16 {
        match addr {
            0x100 | 0x104 | 0x108 | 0x10c => self.timers.get((addr - 0x100) / 4),
            _ => self.reg.load16(addr),
        }
    }

    #[inline]
    fn set(&mut self, addr: u32, val: u16) {
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
            0x202 => self.disable_intrreq(new),
            0x102 | 0x106 | 0x10a | 0x10e => self.timers.updated((addr - 0x102) / 4, old, new),
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
    #[inline]
    fn load8(&self, addr: u32) -> u8 {
        let val = self.get(addr & !1);
        (val >> ((addr & 1) * 8)) as u8
    }

    #[inline]
    fn set8(&mut self, addr: u32, val: u8) {
        let pv = self.get_priv(addr & !1);
        let shift = (addr & 1) * 8;
        let mask = 0xff00 >> shift;
        let nv = (pv & mask) | ((val as u16) << shift);
        self.set(addr & !1, nv);
    }

    #[inline]
    fn load16(&self, addr: u32) -> u16 {
        self.get(addr)
    }

    #[inline]
    fn set16(&mut self, addr: u32, val: u16) {
        self.set(addr, val);
    }

    #[inline]
    fn load32(&self, addr: u32) -> u32 {
        self.get(addr) as u32 | ((self.get(addr + 2) as u32) << 16)
    }

    #[inline]
    fn set32(&mut self, addr: u32, val: u32) {
        // FIXME: there may be issues here with simultaneous assignment rules
        // (specifically on timers?)
        self.set(addr, val as u16);
        self.set(addr + 2, (val >> 16) as u16);
    }
}

/// Mapping from address to a mask representing the write-protected parts
/// of the field
fn ro_mask(addr: u32) -> u16 {
    use std::u16;
    let all = u16::MAX;

    match addr {
        0x130 |
        0x202 => all,
        _ => 0,
    }
}
