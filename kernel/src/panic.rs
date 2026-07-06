//! Panic处理 - 独立模块

use crate::drivers::vga::{VgaWriter, Color};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut vga = VgaWriter::new();
    vga.set_color(Color::Red, Color::Black);
    vga.write_str("\n!!! KERNEL PANIC !!!\n");

    if let Some(location) = info.location() {
        vga.write_str(&alloc::format!(" at {}:{}\n", location.file(), location.line()));
    }

    if let Some(message) = info.message() {
        vga.write_str(&alloc::format!(": {}\n", message));
    }

    loop {}
}
