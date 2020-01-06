use std::default::Default;
use std::sync::{Arc, Mutex};

use arraydeque::{ArrayDeque, Wrapping};
use sdl2::audio::{AudioCallback, AudioSpecDesired};

use mmu::gba::Gba as GbaMmu;
use shared::Shared;

use super::IoReg;

// Sound runs at 32768 Hz
// 256 samples at a time leads to audio latency of ~8ms, which is
// probably ok.

pub const SAMPLES: usize = 256;
pub const FREQ: i32 = 32768;

// do SAMPLES * 4 to give extra buffer room
type SoundDeque = ArrayDeque<[(f32, f32); SAMPLES * 16], Wrapping>;
pub struct SoundBuf(Arc<Mutex<SoundDeque>>);

pub struct Spu<'a> {
    io: Shared<IoReg<'a>>,

    buf: SoundBuf,

    idx: i32,
}

impl<'a> Spu<'a> {
    pub fn new(io: Shared<IoReg<'a>>) -> Self {
        Self {
            io: io,
            buf: Default::default(),
            idx: 0,
        }
    }

    pub fn cycle(&mut self) {
        if self.idx == 0 {
            self.buf.0.lock().unwrap().push_back((1.0, 1.0));
        } else if self.idx == 512 {
            self.buf.0.lock().unwrap().push_back((-1.0, -1.0));
        }
        self.idx = (self.idx + 1) % 1024;
    }

    pub fn get_callback(&self) -> SoundBuf {
        SoundBuf(Arc::clone(&self.buf.0))
    }
}

impl AudioCallback for SoundBuf {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut buf = self.0.lock().unwrap();
        let mut missed = 0;
        warn!("Sound buffer length: {}", buf.len());
        for i in 0..(out.len() / 2) {
            let (l, r) = match buf.pop_front() {
                Some((l, r)) => (l, r),
                None => {
                    //warn!("Sound sample not available when queried");
                    missed += 1;
                    (0.0, 0.0)
                }
            };
            out[i * 2 + 0] = l * 0.5;
            out[i * 2 + 1] = r * 0.5;
        }
        if missed != 0 {
            warn!("Missed {} samples", missed);
        }
    }
}

impl Default for SoundBuf {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Default::default())))
    }
}
