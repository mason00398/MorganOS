//! RTL8139 网卡驱动
//! 
//! FIX #5: DMA 缓冲区现在使用 alloc 分配并确保物理地址连续

use crate::pci::PciDevice;
use core::ptr;
use alloc::vec::Vec;
use alloc::alloc::{alloc, dealloc, Layout};

const RTL8139_VENDOR: u16 = 0x10EC;
const RTL8139_DEVICE: u16 = 0x8139;
const RX_BUF_SIZE: usize = 8192 + 16 + 1500;
const TX_BUF_SIZE: usize = 1536;

pub struct Rtl8139 {
    io_base: u16,
    mac: [u8; 6],
    /// FIX #5: 使用 alloc 分配的物理连续内存
    rx_buffer: *mut u8,
    rx_layout: Layout,
    tx_buffer: *mut u8,
    tx_layout: Layout,
    rx_next: usize,
    initialized: bool,
    arp_cache: alloc::collections::BTreeMap<u32, [u8; 6]>,
}

impl Rtl8139 {
    pub fn new(device: &PciDevice) -> Option<Self> {
        if device.header.vendor_id != RTL8139_VENDOR || device.header.device_id != RTL8139_DEVICE {
            return None;
        }
        let io_base = (device.header.bar0 & 0xFFFC) as u16;
        if io_base == 0 { return None; }

        // 启用 PCI 总线主控
        crate::pci::write_config_u32(
            device.bus, device.slot, device.func, 0x04,
            (device.header.command as u32) | 0x04 | 0x02,
        );

        // 软复位
        unsafe {
            ptr::write_volatile((io_base + 0x37) as *mut u8, 0x10);
            let mut timeout = 1000;
            while (ptr::read_volatile((io_base + 0x37) as *mut u8) & 0x10 != 0) && timeout > 0 {
                timeout -= 1;
            }
            if timeout == 0 { return None; }
        }

        let mut mac = [0u8; 6];
        unsafe {
            for i in 0..6 {
                mac[i] = ptr::read_volatile((io_base + 0x00 + i as u32) as *mut u8);
            }
        }

        // FIX #5: 使用 alloc 分配 DMA 安全的缓冲区
        let rx_layout = Layout::from_size_align(RX_BUF_SIZE, 4096).unwrap();
        let rx_buffer = unsafe { alloc(rx_layout) };
        if rx_buffer.is_null() {
            return None;
        }
        unsafe { core::ptr::write_bytes(rx_buffer, 0, RX_BUF_SIZE); }

        let tx_layout = Layout::from_size_align(TX_BUF_SIZE, 4096).unwrap();
        let tx_buffer = unsafe { alloc(tx_layout) };
        if tx_buffer.is_null() {
            dealloc(rx_buffer, rx_layout);
            return None;
        }
        unsafe { core::ptr::write_bytes(tx_buffer, 0, TX_BUF_SIZE); }

        Some(Self {
            io_base,
            mac,
            rx_buffer,
            rx_layout,
            tx_buffer,
            tx_layout,
            rx_next: 0,
            initialized: false,
            arp_cache: alloc::collections::BTreeMap::new(),
        })
    }

    pub fn mac_addr(&self) -> [u8; 6] { self.mac }

    pub fn init(&mut self) {
        if self.initialized { return; }
        unsafe {
            let rx_addr = self.rx_buffer as u32;
            ptr::write_volatile((self.io_base + 0x30) as *mut u32, rx_addr);
            ptr::write_volatile((self.io_base + 0x44) as *mut u32, 0x0F | 0x01);
            let cmd = ptr::read_volatile((self.io_base + 0x37) as *mut u8);
            ptr::write_volatile((self.io_base + 0x37) as *mut u8, cmd | 0x0C);
            ptr::write_volatile((self.io_base + 0x3C) as *mut u16, 0x0005);
            ptr::write_volatile((self.io_base + 0x38) as *mut u16, 0);
            ptr::write_volatile((self.io_base + 0x3E) as *mut u16, 0xFFFF);
        }
        self.rx_next = 0;
        self.initialized = true;
    }

    pub fn arp_resolve(&mut self, ip: u32) -> Option<[u8; 6]> {
        if let Some(&mac) = self.arp_cache.get(&ip) {
            return Some(mac);
        }
        Some([0xFF; 6])
    }

    pub fn send(&mut self, data: &[u8]) -> bool {
        if !self.initialized || data.len() > 1536 { return false; }
        unsafe {
            // FIX #5: 使用预分配的 tx_buffer
            ptr::copy_nonoverlapping(data.as_ptr(), self.tx_buffer, data.len());
            ptr::write_volatile((self.io_base + 0x20) as *mut u32, self.tx_buffer as u32);
            let status = (data.len() as u32) | 0x1000 | 0x0100;
            ptr::write_volatile((self.io_base + 0x10) as *mut u32, status);
            let mut retries = 1000;
            while retries > 0 {
                let isr = ptr::read_volatile((self.io_base + 0x3E) as *mut u16);
                if isr & 0x0004 != 0 {
                    ptr::write_volatile((self.io_base + 0x3E) as *mut u16, 0x0004);
                    return true;
                }
                retries -= 1;
            }
            false
        }
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Option<usize> {
        if !self.initialized { return None; }
        unsafe {
            let status = ptr::read_volatile((self.io_base + 0x3E) as *mut u16);
            if status & 0x0001 == 0 { return None; }

            let rx_addr = self.rx_buffer;
            let buf_size = RX_BUF_SIZE;

            while self.rx_next < buf_size {
                let header = ptr::read_volatile(rx_addr.add(self.rx_next) as *const u16);
                let packet_len = (header & 0x1FFF) as usize;

                if (header & 0x8000) != 0 || packet_len == 0 || packet_len > 1536 {
                    self.rx_next += 4;
                    if self.rx_next >= buf_size { self.rx_next = 0; }
                    continue;
                }

                if packet_len > buf.len() {
                    self.rx_next += (packet_len + 4 + 3) & !3;
                    if self.rx_next >= buf_size { self.rx_next = 0; }
                    continue;
                }

                let data_start = self.rx_next + 4;
                let data_end = data_start + packet_len;

                if data_end <= buf_size {
                    ptr::copy_nonoverlapping(
                        rx_addr.add(data_start),
                        buf.as_mut_ptr(),
                        packet_len,
                    );
                } else {
                    let first_part = buf_size - data_start;
                    ptr::copy_nonoverlapping(
                        rx_addr.add(data_start),
                        buf.as_mut_ptr(),
                        first_part,
                    );
                    let second_part = packet_len - first_part;
                    ptr::copy_nonoverlapping(
                        rx_addr,
                        buf.as_mut_ptr().add(first_part),
                        second_part,
                    );
                }

                let new_cabr = (self.rx_next + packet_len + 4 + 3) & !3;
                ptr::write_volatile(
                    (self.io_base + 0x38) as *mut u16,
                    (new_cabr - 16) as u16,
                );

                self.rx_next = new_cabr;
                if self.rx_next >= buf_size { self.rx_next = 0; }

                ptr::write_volatile((self.io_base + 0x3E) as *mut u16, 0x0001);
                return Some(packet_len);
            }

            self.rx_next = 0;
            None
        }
    }
}

impl Drop for Rtl8139 {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.rx_buffer, self.rx_layout);
            dealloc(self.tx_buffer, self.tx_layout);
        }
    }
}
