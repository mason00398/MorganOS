//! AHCI 存储驱动 - 直接寄存器操作（不依赖外部 crate）
//! 
//! FIX #3: Replaced simple-ahci dependency with direct register access
//! simple-ahci crate does not exist on crates.io
//! This implementation uses standard AHCI register interface

use core::ptr;
use alloc::vec::Vec;
use crate::pci::PciDevice;

/// AHCI 寄存器偏移
const AHCI_BAR5_OFFSET: u32 = 0x04;
const AHCI_GLOBAL_HBA_PORT: u32 = 0x100;
const AHCI_PORT_IRQ_STAT: u32 = 0x28;
const AHCI_PORT_CMD_ST: u32 = 0x02;
const AHCI_PORT_CMD_PI: u32 = 0x04;

/// FIS 数据结构
#[repr(C, packed)]
struct FISRegHost {
    fis_type: u8,           // 0x27
    port_i: u8,             // port multiplier
    command: u8,            // command register
    feature_l: u8,          // feature register (low)
    lba0: u8,               // LBA low
    lba1: u8,               // LBA mid
    lba2: u8,               // LBA high
    device: u8,             // device register
    lba3: u8,               // LBA low extended
    lba4: u8,               // LBA mid extended
    lba5: u8,               // LBA high extended
    feature_h: u8,          // feature register (high)
    count: u16,             // sector count
    icc: u8,                // interrupt completion
    control: u8,            // control register
}

impl FISRegHost {
    fn new(lba: u64, count: u16, write: bool) -> Self {
        Self {
            fis_type: 0x27,
            port_i: 0,
            command: if write { 0x30 } else { 0x20 }, // WRITE or READ DMA
            feature_l: (lba & 0xFF) as u8,
            lba0: ((lba >> 8) & 0xFF) as u8,
            lba1: ((lba >> 16) & 0xFF) as u8,
            lba2: ((lba >> 24) & 0xFF) as u8,
            device: 0x40, // LBA mode
            lba3: ((lba >> 32) & 0xFF) as u8,
            lba4: ((lba >> 40) & 0xFF) as u8,
            lba5: ((lba >> 48) & 0xFF) as u8,
            feature_h: ((lba >> 56) & 0xFF) as u8,
            count,
            icc: 0,
            control: 0,
        }
    }
}

/// AHCI 端口
struct AhciPort {
    base: *mut u8,
}

impl AhciPort {
    fn is_ready(&self) -> bool {
        unsafe { (ptr::read_volatile(self.base.offset(0x04) as *const u32) & 0x01) != 0 }
    }
    
    fn read_sector(&self, lba: u64, count: u16, buf: &mut [u8]) -> bool {
        self.transfer(lba, count, buf, false)
    }
    
    fn write_sector(&self, lba: u64, count: u16, buf: &[u8]) -> bool {
        self.transfer(lba, count, buf, true)
    }
    
    fn transfer(&self, lba: u64, count: u16, buf: &[u8], write: bool) -> bool {
        if buf.len() < (count as usize) * 512 {
            return false;
        }
        
        // 准备 FIS
        let fis = FISRegHost::new(lba, count, write);
        let fis_ptr = buf.as_ptr() as *const FISRegHost;
        
        // 发送命令（简化版，实际需要 HBA 接口）
        // 这里假设 DMA 缓冲区已准备好
        unsafe {
            ptr::write_volatile(self.base.offset(0x40) as *mut u64, fis_ptr as u64);
            let cmd = ptr::read_volatile(self.base.offset(0x04) as *const u32);
            ptr::write_volatile(self.base.offset(0x04) as *mut u32, cmd | 0x01); // Start
        }
        
        // 等待完成
        let mut timeout = 10000;
        while timeout > 0 {
            unsafe {
                let irq = ptr::read_volatile(self.base.offset(0x28) as *const u32);
                if (irq & 0x01) != 0 {
                    ptr::write_volatile(self.base.offset(0x28) as *mut u32, irq); // Clear IRQ
                    return true;
                }
                timeout -= 1;
            }
        }
        false
    }
}

pub struct AhciManager {
    ports: Vec<AhciPort>,
    port_count: u8,
    abar_base: u64,
}

impl AhciManager {
    pub fn new(device: &PciDevice) -> Option<Self> {
        let abar = device.header.bar5 & 0xFFFFFFF0;
        if abar == 0 { return None; }
        
        // 启用 PCI 总线主控
        unsafe {
            let cmd = ptr::read_volatile((abar + 0x04) as *const u16);
            ptr::write_volatile((abar + 0x04) as *mut u16, cmd | 0x04 | 0x02);
        }
        
        // 软重置
        unsafe {
            ptr::write_volatile((abar + 0x04) as *mut u8, 0x01); // Global Reset
            let mut timeout = 1000;
            while timeout > 0 {
                let status = ptr::read_volatile((abar + 0x04) as *const u8);
                if (status & 0x01) == 0 { break; }
                timeout -= 1;
            }
            if timeout == 0 { return None; }
        }
        
        let mut ports = Vec::new();
        let mut port_count = 0;
        
        // 检查支持的端口数
        unsafe {
            let cap = ptr::read_volatile((abar + AHCI_GLOBAL_HBA_PORT) as *const u32);
            let nports = ((cap >> 8) & 0x1F) as u8;
            
            for i in 0..nports {
                let port_reg = abar + 0x100 + (i as u64) * 0x80;
                let cmd = ptr::read_volatile((port_reg + 0x04) as *const u32);
                if (cmd & 0x01) != 0 { // Port implemented
                    ports.push(AhciPort { base: (port_reg) as *mut u8 });
                    port_count += 1;
                }
            }
        }
        
        if ports.is_empty() { return None; }
        
        Some(Self { ports, port_count, abar_base: abar as u64 })
    }
    
    pub fn port_count(&self) -> u8 { self.port_count }
    
    pub fn read_sectors(&self, port_idx: u8, lba: u64, count: u8, buf: &mut [u8]) -> bool {
        if port_idx as usize >= self.ports.len() { return false; }
        let port = &self.ports[port_idx as usize];
        port.read_sector(lba, count as u16, buf)
    }
    
    pub fn write_sectors(&self, port_idx: u8, lba: u64, count: u8, buf: &[u8]) -> bool {
        if port_idx as usize >= self.ports.len() { return false; }
        let port = &self.ports[port_idx as usize];
        port.write_sector(lba, count as u16, buf)
    }
}
