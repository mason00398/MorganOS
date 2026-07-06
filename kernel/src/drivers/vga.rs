use core::fmt;
use core::ptr;
use volatile::Volatile;

const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black = 0, Blue = 1, Green = 2, Cyan = 3,
    Red = 4, Magenta = 5, Brown = 6, LightGray = 7,
    DarkGray = 8, LightBlue = 9, LightGreen = 10,
    LightCyan = 11, LightRed = 12, Pink = 13,
    Yellow = 14, White = 15,
}

#[repr(transparent)]
struct VgaChar(u16);

impl VgaChar {
    #[inline]
    fn new(ch: u8, color: u8) -> Self {
        VgaChar((color as u16) << 8 | ch as u16)
    }
}

pub struct VgaWriter {
    col: usize,
    row: usize,
    color: u8,
    buffer: &'static mut [Volatile<u16>],
}

impl VgaWriter {
    #[inline]
    pub fn new() -> Self {
        Self {
            col: 0, row: 0, color: 0x0F,
            buffer: unsafe {
                core::slice::from_raw_parts_mut(
                    VGA_BUFFER as *mut Volatile<u16>,
                    VGA_WIDTH * VGA_HEIGHT
                )
            },
        }
    }

    #[inline]
    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = (bg as u8) << 4 | (fg as u8);
    }

    #[inline]
    pub fn get_color(&self) -> u8 {
        self.color
    }

    /// 真正的 VGA 清屏
    pub fn clear(&mut self) {
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            self.buffer[i].write(VgaChar::new(b' ', self.color).0);
        }
        self.col = 0;
        self.row = 0;
    }

    #[inline]
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => { self.col = 0; self.row += 1; }
            b'\r' => { self.col = 0; }
            b'\t' => {
                let spaces = 4 - (self.col % 4);
                for _ in 0..spaces { self.write_byte(b' '); }
            }
            _ => {
                let idx = self.row * VGA_WIDTH + self.col;
                if idx < VGA_WIDTH * VGA_HEIGHT {
                    self.buffer[idx].write(VgaChar::new(byte, self.color).0);
                }
                self.col += 1;
                if self.col >= VGA_WIDTH {
                    self.col = 0;
                    self.row += 1;
                }
            }
        }
        if self.row >= VGA_HEIGHT {
            self.scroll();
        }
    }

    pub fn scroll(&mut self) {
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let src = row * VGA_WIDTH + col;
                let dst = (row - 1) * VGA_WIDTH + col;
                self.buffer[dst].write(self.buffer[src].read());
            }
        }
        for col in 0..VGA_WIDTH {
            let idx = (VGA_HEIGHT - 1) * VGA_WIDTH + col;
            self.buffer[idx].write(VgaChar::new(b' ', self.color).0);
        }
        self.row = VGA_HEIGHT - 1;
    }

    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }

    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        struct Writer<'a>(&'a mut VgaWriter);
        impl fmt::Write for Writer<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0.write_str(s);
                Ok(())
            }
        }
        if let Ok(mut w) = Writer::new(self) {
            let _ = w.write_fmt(args);
        }
    }

    /// 设置光标位置
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    /// 获取当前行
    #[inline]
    pub fn get_row(&self) -> usize {
        self.row
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
