use bit_util::{bit, extract};
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use mmu::Mmu;
use shared::Shared;

use super::IoReg;

const TIMERS: usize = 4;

#[derive(Copy, Clone, Default)]
pub struct Timers<'a> {
    timers: [u16; TIMERS],
    cycles: u64,
    io: Shared<IoReg<'a>>,
}

impl<'a> Serialize for Timers<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("gba_rs::io::timer::Timers", 2)?;
        s.serialize_field("timers", &self.timers)?;
        s.serialize_field("cycles", &self.cycles)?;
        s.end()
    }
}

impl<'a> Timers<'a> {
    pub fn init(&mut self, io: Shared<IoReg<'a>>) {
        self.io = io;
    }

    /// Increments each timer,
    /// and raises relevant interrupts
    pub fn cycle(&mut self) {
        let mut res = false;
        for i in 0..TIMERS {
            let ctrl = self.io.reg.load32(0x100 + 4 * i as u32);
            let ret = self.timers[i].cycle(ctrl, self.cycles as u16, res);
            res = ret.0;
            if ret.1 {
                self.io.raise_interrupt(3 + i as u8);
            }
        }
        self.cycles += 1;
    }

    pub fn updated(&mut self, idx: u32, old: u16, new: u16) {
        debug_assert!(idx <= TIMERS as u32);

        if bit(old as u32, 7) == 0 && bit(new as u32, 7) == 1 {
            self.timers[idx as usize] = self.io.get_priv(0x100 + 4 * idx);
        }
    }

    pub fn get(&self, idx: u32) -> u16 {
        debug_assert!(idx <= TIMERS as u32);
        self.timers[idx as usize]
    }
}

trait Timer {
    fn cycle(&mut self, ctrl: u32, cycle: u16, cascade: bool) -> (bool, bool);
}

impl Timer for u16 {
    /// Increments the timer by one
    /// Returns if the timer overflowed and whether an interrupt is requested
    /// ctrl is both reset and control values
    fn cycle(&mut self, ctrl: u32, cycle: u16, cascade: bool) -> (bool, bool) {
        if bit(ctrl, 23) == 0 {
            return (false, false);
        };

        let inc = if bit(ctrl, 18) == 0 {
            let mask = match extract(ctrl, 16, 2) {
                0 => 0,
                1 => 63,
                2 => 255,
                3 => 1023,
                _ => unreachable!(),
            };
            cycle & mask == 0
        } else {
            cascade
        };

        if inc {
            *self = self.wrapping_add(1);
            let overflow = *self == 0;
            if overflow {
                *self = ctrl as u16;
            }
            (overflow, overflow && bit(ctrl, 22) == 1)
        } else {
            (false, false)
        }
    }
}
