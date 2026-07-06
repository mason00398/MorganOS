use crate::console::Shell;
use core::ptr;

pub fn run(shell: &mut Shell) {
    shell.write_str("Rebooting...\n");
    unsafe {
        // 通过键盘控制器复位
        let mut timeout = 50000;
        ptr::write_volatile(0x64 as *mut u8, 0xFE);
        while timeout > 0 {
            timeout -= 1;
        }
    }
    // 备用：ACPI 重启
    loop {}
}
