use bit_util::{bit, extract};

use mmu::Mmu;
use mmu::gba::Gba as GbaMmu;
use shared::Shared;

use super::IoReg;

const CHANNELS: usize = 4;

#[derive(Copy, Clone, Default)]
pub struct Dma<'a> {
    chs: [DmaCh; CHANNELS],
    io: Shared<IoReg<'a>>,
}

#[derive(Copy, Clone, Default)]
struct DmaCh {
    sad: u32,
    dad: u32,
    len: u32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Trigger {
    HBlank,
    VBlank,
    SoundFifo,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Register {
    Source,
    Dest,
}

fn addr_bits(reg: Register, ch: usize) -> u32 {
    match reg {
        Register::Source => match ch {
            0 => 27,
            1 | 2 | 3 => 28,
            _ => unreachable!(),
        },
        Register::Dest => match ch {
            0 | 1 | 2 => 27,
            3 => 28,
            _ => unreachable!(),
        },
    }
}

impl<'a> Dma<'a> {
    pub fn init(&mut self, io: Shared<IoReg<'a>>) {
        self.io = io;
    }

    pub fn updated(&mut self, addr: u32, old: u16, val: u16) {
        let channel = (addr / 12) as usize;
        debug_assert!(channel < 4);

        let reg = addr % 12;
        if reg == 0xA {
            if bit(old as u32, 15) == 0 && bit(val as u32, 15) == 1 {
                self.refresh(channel, val, false);
                if extract(val as u32, 12, 2) == 0 {
                    self.start(channel, val);
                }
            }
        }
    }

    pub fn trigger(&mut self, trigger: Trigger) {
        for ch in 0..CHANNELS {
            let ctrl = self.io.get_priv(0xBA + 12 * ch as u32) as u32;
            if bit(ctrl, 15) == 0 || bit(ctrl, 9) == 0 {
                continue;
            }
            let timing = extract(ctrl, 12, 2);
            let run = match trigger {
                Trigger::HBlank => timing == 2,
                Trigger::VBlank => timing == 1,
                Trigger::SoundFifo => timing == 3 && (ch == 1 || ch == 2),
            };
            if run {
                self.refresh(ch, ctrl as u16, true);
                self.start(ch, ctrl as u16);
            }
        }
    }

    fn refresh(&mut self, ch: usize, ctrl: u16, repeat: bool) {
        debug_assert!(ch < 4);
        let base = 0xB0 + 12 * ch as u32;
        self.chs[ch].set_count(self.io.get_priv(base + 8), ch);
        if !repeat || extract(ctrl as u32, 5, 2) == 3 {
            self.chs[ch].set_addr(Register::Dest, ch,
                self.io.reg.load32(base + 4));
        }
        if !repeat {
            self.chs[ch].set_addr(Register::Source, ch,
                self.io.reg.load32(base));
        }
    }

    fn start(&mut self, ch: usize, ctrl: u16) {
        debug_assert!(ch < 4);
        let base = 0xB0 + 12 * ch as u32;

        do_copy(&mut self.chs[ch], &mut self.io.mmu, ctrl);

        if bit(ctrl as u32, 9) == 0 {
            let nctrl = ctrl & !(1 << 15);
            self.io.set_priv(base + 10, nctrl);
        }
        if bit(ctrl as u32, 14) == 1 {
            self.io.raise_interrupt(8 + ch as u8);
        }
    }
}

fn do_copy<'a>(regs: &mut DmaCh, mmu: &mut GbaMmu<'a>, ctrl: u16) {
    let ctrl = ctrl as u32;
    let halfword = bit(ctrl, 10) == 0;
    let word = if halfword { 2 } else { 4 };
    let dinc = match extract(ctrl, 5, 2) {
        0 | 3 => word,
        1 => 0u32.wrapping_sub(word),
        2 => 0,
        _ => unreachable!(),
    };
    let sinc = match extract(ctrl, 7, 2) {
        0 => word,
        1 => 0u32.wrapping_sub(word),
        2 => 0,
        3 => { warn!("Invalid source increment setting"); word },
        _ => unreachable!(),
    };

    regs.sad &= !(word-1);
    // FIXME: DMA copying from BIOS memory should write 0's when not executing
    // BIOS code
    for _ in 0..regs.len {
        if halfword {
            let val = mmu.load16(regs.sad);
            mmu.set16(regs.dad, val);
        } else {
            let val = mmu.load32(regs.sad);
            mmu.set32(regs.dad, val);
        }
        regs.dad = regs.dad.wrapping_add(dinc);
        regs.sad = regs.sad.wrapping_add(sinc);
    }
}

impl DmaCh {
    fn set_addr(&mut self, reg: Register, channel: usize, val: u32) {
        let bits = addr_bits(reg, channel);
        let nval = val & ((1 << bits) - 1);
        match reg {
            Register::Source => self.sad = nval,
            Register::Dest => self.dad = nval,
        }
    }

    fn set_count(&mut self, val: u16, channel: usize) {
        let max = if channel == 3 { 0x10000 } else { 0x4000 };
        let mval = (val as u32) & (max - 1);

        self.len = if mval == 0 { max } else { mval };
    }
}
