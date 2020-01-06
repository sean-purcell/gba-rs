use sdl2::keyboard::KeyboardState;

use bit_util::bit;

use super::{IoReg, KEYCNT, KEYINPUT};

pub struct KeyState {
    a: bool,
    b: bool,
    select: bool,
    start: bool,
    r: bool,
    l: bool,
    u: bool,
    d: bool,
    br: bool,
    bl: bool,
}

impl KeyState {
    // TODO: abstract away key selections, use a trait here
    pub fn new_from_keystate(state: &KeyboardState) -> Self {
        use sdl2::keyboard::Scancode::*;
        KeyState {
            a: state.is_scancode_pressed(L),
            b: state.is_scancode_pressed(K),
            select: state.is_scancode_pressed(Z),
            start: state.is_scancode_pressed(X),
            r: state.is_scancode_pressed(D),
            l: state.is_scancode_pressed(A),
            u: state.is_scancode_pressed(W),
            d: state.is_scancode_pressed(S),
            br: state.is_scancode_pressed(P),
            bl: state.is_scancode_pressed(I),
        }
    }
}

impl<'a> IoReg<'a> {
    pub fn set_keyreg(&mut self, state: &KeyState) {
        let vals = ((state.a as u16) << 0)
            | ((state.b as u16) << 1)
            | ((state.select as u16) << 2)
            | ((state.start as u16) << 3)
            | ((state.r as u16) << 4)
            | ((state.l as u16) << 5)
            | ((state.u as u16) << 6)
            | ((state.d as u16) << 7)
            | ((state.br as u16) << 8)
            | ((state.bl as u16) << 9);
        let reg = !vals & 0x3ff;
        self.set_priv(KEYINPUT, reg);

        let keycnt = self.get_priv(KEYCNT);
        self.check_key_intr(reg, keycnt);
    }

    pub(super) fn check_key_intr(&mut self, keyinput: u16, keycnt: u16) {
        if bit(keycnt as u32, 14) == 1 {
            let mask = keycnt & 1023;
            let intr = if bit(keycnt as u32, 15) == 0 {
                keyinput & mask != 0
            } else {
                keyinput & mask == mask
            };
            if intr {
                self.raise_interrupt(12);
            }
        }
    }
}
