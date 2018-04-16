use std::cell::{RefCell, RefMut};
use std::rc::{Rc, Weak};

use sdl2::render::Canvas;
use sdl2::video::Window;

use mmu::ram::Ram;
use shared::Shared;

use super::IoReg;

/// Handle scanline drawing here
pub struct Ppu {
    canvas: Shared<Canvas<Window>>,
    col: u32,
    row: u32,
}

impl Ppu {
    pub fn new(canvas: &Rc<RefCell<Canvas<Window>>>) -> Ppu {
        Ppu {
            canvas: Rc::downgrade(canvas),
            io: Weak::new(),
            vram: Weak::new(),
            col: 0,
            row: 0,
        }
    }

    pub fn set_io(&mut self, io: &Rc<RefCell<IoReg>>) {
        self.io = Rc::downgrade(io);
    }

    pub fn set_vram(&mut self, vram: &Rc<RefCell<Ram>>) {
        self.vram = Rc::downgrade(vram);
    }

    pub fn cycle(&mut self) {
        // Want to render pixel at col, row
    }
}
