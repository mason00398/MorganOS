use core::ptr;
use alloc::format;

pub struct Rtc;

impl Rtc {
    fn read_register(reg: u8) -> u8 {
        unsafe {
            ptr::write_volatile(0x70 as *const u8, reg | 0x80);
            core::hint::spin_loop();
            ptr::read_volatile(0x71 as *const u8)
        }
    }

    fn is_updating() -> bool {
        unsafe {
            ptr::write_volatile(0x70 as *const u8, 0x0A);
            ptr::read_volatile(0x71 as *const u8) & 0x80 != 0
        }
    }

    fn bcd_to_bin(bcd: u8) -> u8 {
        (bcd >> 4) * 10 + (bcd & 0x0F)
    }

    pub fn get_time() -> (u8, u8, u8, u8, u8, u8) {
        // 等待 RTC 更新完成
        while Self::is_updating() {
            // spin
        }

        let second = Self::read_register(0x00);
        let minute = Self::read_register(0x02);
        let hour = Self::read_register(0x04);
        let day = Self::read_register(0x07);
        let month = Self::read_register(0x08);
        let year = Self::read_register(0x09);

        let status_b = Self::read_register(0x0B);
        if status_b & 0x04 != 0 {
            (second, minute, hour, day, month, year)
        } else {
            (
                Self::bcd_to_bin(second),
                Self::bcd_to_bin(minute),
                Self::bcd_to_bin(hour),
                Self::bcd_to_bin(day),
                Self::bcd_to_bin(month),
                Self::bcd_to_bin(year),
            )
        }
    }

    pub fn get_datetime_string() -> alloc::string::String {
        let (sec, min, hour, day, month, year) = Self::get_time();
        format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            2000 + year as u32, month, day, hour, min, sec)
    }

    pub fn get_uptime_seconds() -> u64 {
        // 简单实现：从启动开始计数（需要在 main.rs 中维护）
        // 这里用 RTC 秒数作为近似
        let (sec, _, _, _, _, _) = Self::get_time();
        sec as u64
    }
}
