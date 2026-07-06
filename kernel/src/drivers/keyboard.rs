//! PS/2键盘驱动 - 基于pc-keyboard
//! 
//! FIX #6: Added direction key and modifier support

use core::ptr;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts::Us104Key, DecodedKey, HandleControl};
use spin::Mutex;

static KEYBOARD: Mutex<Option<Keyboard<Us104Key, ScancodeSet1>>> = Mutex::new(None);
static KEY_BUFFER: Mutex<alloc::collections::VecDeque<char>> = Mutex::new(alloc::collections::VecDeque::new());

pub fn init() {
    let mut kb = KEYBOARD.lock();
    if kb.is_none() {
        *kb = Some(Keyboard::new(Us104Key::new(), ScancodeSet1::new()));
    }

    unsafe {
        ptr::write_volatile(0x64 as *mut u8, 0xAE);
        while ptr::read_volatile(0x64 as *const u8) & 0x01 != 0 {
            let _ = ptr::read_volatile(0x60 as *const u8);
        }
        ptr::write_volatile(0x60 as *mut u8, 0xF4);
    }
}

pub fn key_pressed() -> bool {
    unsafe { (ptr::read_volatile(0x64 as *const u8) & 0x01) != 0 }
}

/// 读取字符 - 支持方向键、Shift、Ctrl 等修饰键
pub fn read_char() -> Option<char> {
    {
        let mut buf = KEY_BUFFER.lock();
        if let Some(c) = buf.pop_front() {
            return Some(c);
        }
    }

    let mut kb = KEYBOARD.lock();
    let kb = match kb.as_mut() {
        Some(k) => k,
        None => return None,
    };

    unsafe {
        let status = ptr::read_volatile(0x64 as *const u8);
        if status & 0x01 != 0 {
            let scancode = ptr::read_volatile(0x60 as *const u8);
            if let Ok(Some(event)) = kb.add_byte(scancode) {
                if let Some(key) = kb.process_keyevent_with(event, HandleControl::Ignore) {
                    match key {
                        DecodedKey::Unicode(c) => {
                            let mut buf = KEY_BUFFER.lock();
                            buf.push_back(c);
                            return Some(c);
                        }
                        DecodedKey::RawKey(_) => {
                            // 方向键、Home、End 等键 - 可以扩展处理
                        }
                    }
                }
            }
        }
        None
    }
}

/// 读取原始扫描码（用于方向键处理）
pub fn read_scancode() -> Option<u8> {
    unsafe {
        let status = ptr::read_volatile(0x64 as *const u8);
        if status & 0x01 != 0 {
            Some(ptr::read_volatile(0x60 as *const u8))
        } else {
            None
        }
    }
}
