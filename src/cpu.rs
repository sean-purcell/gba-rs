use arm7tdmi_rs::{exception::Exception, reg, reg::Reg, Cpu as Arm7TDMICpu, Memory};
use shared::*;

use mmu::MemoryUnit;

pub use arm7tdmi_rs::exception;

#[derive(Serialize, Deserialize)]
pub struct Cpu<T: MemoryUnit> {
    cpu: Arm7TDMICpu,
    #[serde(skip, default = "Default::default")]
    mmu: Option<MemWrapper<Shared<T>>>,
}

struct MemWrapper<T>(T);

impl<T: MemoryUnit> Memory for MemWrapper<Shared<T>> {
    fn r8(&mut self, addr: u32) -> u8 {
        self.0.load8(addr)
    }
    fn r16(&mut self, addr: u32) -> u16 {
        self.0.load16(addr)
    }
    fn r32(&mut self, addr: u32) -> u32 {
        self.0.load32(addr)
    }
    fn w8(&mut self, addr: u32, val: u8) {
        self.0.set8(addr, val)
    }
    fn w16(&mut self, addr: u32, val: u16) {
        self.0.set16(addr, val)
    }
    fn w32(&mut self, addr: u32, val: u32) {
        self.0.set32(addr, val)
    }
}

impl<T: MemoryUnit> Cpu<T> {
    pub fn new<'a, I>(mmu: Shared<T>, regs: I) -> Self
    where
        I: IntoIterator<Item = &'a (usize, Reg, u32)>,
    {
        Cpu {
            cpu: (Arm7TDMICpu::new(regs)),
            mmu: Some(MemWrapper(mmu)),
        }
    }

    /// Initializes registers according to the ARM documentation
    pub fn init_arm(&mut self) {
        // http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.faqs/ka3761.html
        self.init(&[(0, reg::PC, 0), (0, reg::CPSR, 0xd3)]);
    }

    /// Initializes the registers to emulate booting through BIOS, to directly
    /// start a ROM
    pub fn init_direct(&mut self) {
        self.init(&[
            (0, reg::PC, 0x8000000),
            (0, reg::CPSR, 0x1f),
            (0, reg::SP, 0x3007f00),
            (2, reg::SP, 0x3007fa0),
            (3, reg::SP, 0x3007fe0),
        ]);
    }

    fn init<'a, I>(&mut self, regs: I)
    where
        I: IntoIterator<Item = &'a (usize, Reg, u32)>,
    {
        self.cpu = Arm7TDMICpu::new(regs)
    }

    pub fn set_breaks<'a, I>(&mut self, brks: I)
    where
        I: IntoIterator<Item = &'a u32>,
    {
        self.cpu.set_breaks(brks);
    }

    pub fn cycle(&mut self) -> bool {
        self.cpu.cycle(self.mmu.as_mut().unwrap())
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn exception(&mut self, exc: &Exception) {
        self.cpu.exception(*exc)
    }

    pub fn irq_enable(&self) -> bool {
        self.cpu.irq_enable()
    }

    pub fn thumb_mode(&self) -> bool {
        self.cpu.thumb_mode()
    }

    pub fn get_prefetch_addr(&self) -> u32 {
        self.cpu.get_prefetch_addr()
    }
}
