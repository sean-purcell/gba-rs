use mmu::MemoryUnit;

use super::Cpu;

impl<T: MemoryUnit> Cpu<T> {
    pub(super) fn load8(&self, addr: u32, seq: bool, cycles: &mut i32) -> u32 {
        cycles += self.mmu.get_cycles(addr, 1, seq);
        self.mmu.load8(addr) as u32
    }

    pub(super) fn set8(&mut self, addr: u32, val: u8, seq: bool, cycles: &mut i32) {
        self.mmu.set8(addr & !1, val);
        cycles += self.mmu.get_cycles(addr, 1, seq);
    }

    pub(super) fn load16(&self, addr: u32, seq: bool, cycles: &mut i32) -> u32 {
        let a = addr & !1;

        cycles += self.mmu.get_cycles(a, 2, seq);

        let val = self.mmu.load16(a);
        bit_util::shift_ror(val as u32, (addr & 1) * 8).0
    }

    pub(super) fn set16(&mut self, addr: u32, val: u16, seq: bool, cycles: &mut i32) {
        self.mmu.set16(addr & !1, val);
        cycles += self.mmu.get_cycles(addr, 2, seq);
    }

    // FIXME: other sizes should likely also have
    // centralized behaviour for unaligned access
    pub(super) fn load32(&self, addr: u32, seq: bool, cycles: &mut i32) -> u32 {
        let a = addr & !3;

        cycles += self.mmu.get_cycles(a, 4, seq);

        let val = self.mmu.load32(a);
        if a == addr {
            val
        } else {
            let shift = (addr & 3) * 8;
            val.rotate_right(shift)
        }
    }

    pub(super) fn set32(&mut self, addr: u32, val: u32, seq: bool, cycles: &mut i32) {
        self.mmu.set32(addr & !3, val);
        cycles += self.mmu.get_cycles(addr, 4, seq);
    }
}
