use byteorder::{ByteOrder, LittleEndian};

use mmu::MemoryRead;

pub fn load8(arr: &[u8], addr: u32) -> MemoryRead<u8> {
    use self::MemoryRead::*;

    let idx = addr as usize;
    if idx < arr.len() {
        Value(arr[idx])
    } else {
        Open
    }
}

pub fn set8(arr: &mut [u8], addr: u32, val: u8) {
    let idx = addr as usize;
    if idx < arr.len() {
        arr[idx] = val;
    }
}

pub fn load16(arr: &[u8], addr: u32) -> MemoryRead<u16> {
    debug_assert!(addr % 2 == 0);

    use self::MemoryRead::*;

    let idx = addr as usize;
    if idx < arr.len() - 1 {
        Value(LittleEndian::read_u16(&arr[idx..idx + 2]))
    } else {
        Open
    }
}

pub fn set16(arr: &mut [u8], addr: u32, val: u16) {
    debug_assert!(addr % 2 == 0);

    let idx = addr as usize;
    if idx < arr.len() - 1 {
        LittleEndian::write_u16(&mut arr[idx..idx + 2], val);
    }
}

pub fn load32(arr: &[u8], addr: u32) -> MemoryRead<u32> {
    debug_assert!(addr % 4 == 0);

    use self::MemoryRead::*;

    let idx = addr as usize;
    if idx < arr.len() - 3 {
        Value(LittleEndian::read_u32(&arr[idx..idx + 4]))
    } else {
        Open
    }
}

pub fn set32(arr: &mut [u8], addr: u32, val: u32) {
    debug_assert!(addr % 4 == 0);

    let idx = addr as usize;
    if idx < arr.len() - 3 {
        LittleEndian::write_u32(&mut arr[idx..idx + 4], val);
    }
}
