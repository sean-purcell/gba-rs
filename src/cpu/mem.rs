use mmu::Mmu;

use super::Cpu;

impl<T: Mmu> Cpu<T> {
    // FIXME: other sizes should likely also have
    // centralized behaviour for unaligned access
    pub(super) fn load32(&self, addr: u32) -> u32 {
        let a = addr & !3;

        let val = self.mmu.load32(a);
        if a == addr {
            val
        } else {
            let shift = (addr & 3) * 8;
            val.rotate_right(shift)
        }
    }
}
