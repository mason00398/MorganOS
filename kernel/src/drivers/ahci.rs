//! AHCI存储驱动 - 基于simple-ahci

use core::ptr;
use alloc::vec::Vec;
use crate::pci::PciDevice;
use simple_ahci::{AhciDriver as SimpleAhci, AhciPort};

pub struct AhciManager {
    driver: SimpleAhci,
    ports: Vec<AhciPort>,
    port_count: u8,
}

impl AhciManager {
    pub fn new(device: &PciDevice) -> Option<Self> {
        let abar = device.header.bar5 & 0xFFFFFFF0;
        if abar == 0 { return None; }

        crate::pci::write_config_u32(
            device.bus, device.slot, device.func, 0x04,
            (device.header.command as u32) | 0x04 | 0x02,
        );

        let mut driver = unsafe { SimpleAhci::new(abar as *mut ())? };
        let mut ports = Vec::new();
        let mut port_count = 0;

        for i in 0..32 {
            if let Some(port) = driver.port(i) {
                if port.is_present() {
                    ports.push(port);
                    port_count += 1;
                }
            }
        }

        if ports.is_empty() { return None; }

        Some(Self { driver, ports, port_count })
    }

    pub fn port_count(&self) -> u8 { self.port_count }

    // ===== 修复 #1：使用simple-ahci的identify =====
    pub fn identify(&mut self, port_idx: u8) -> Option<simple_ahci::IdentifyDevice> {
        if port_idx as usize >= self.ports.len() { return None; }
        let port = &mut self.ports[port_idx as usize];
        port.identify().ok()
    }

    // ===== 修复 #4,#5：带边界检查的读写 =====
    pub fn read_sectors(&mut self, port_idx: u8, lba: u64, count: u8, buf: &mut [u8]) -> bool {
        if port_idx as usize >= self.ports.len() { return false; }
        let need = (count as usize) * 512;
        if buf.len() < need { return false; }
        let port = &mut self.ports[port_idx as usize];
        port.read(lba, count, buf).is_ok()
    }

    pub fn write_sectors(&mut self, port_idx: u8, lba: u64, count: u8, buf: &[u8]) -> bool {
        if port_idx as usize >= self.ports.len() { return false; }
        let need = (count as usize) * 512;
        if buf.len() < need { return false; }
        let port = &mut self.ports[port_idx as usize];
        port.write(lba, count, buf).is_ok()
    }
}
