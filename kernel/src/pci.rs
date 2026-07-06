use alloc::vec::Vec;
use core::ptr;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PciConfigHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,
    pub cardbus_cis_ptr: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_base_addr: u32,
    pub capabilities_ptr: u8,
    pub reserved0: [u8; 3],
    pub reserved1: u32,
    pub reserved2: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub header: PciConfigHeader,
}

pub fn read_config_u8(bus: u8, slot: u8, func: u8, offset: u8) -> u8 {
    let address = (0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)) as u32;
    unsafe {
        ptr::write_volatile(0xCF8 as *mut u32, address);
        ptr::read_volatile(0xCFC as *mut u8)
    }
}

pub fn read_config_u16(bus: u8, slot: u8, func: u8, offset: u8) -> u16 {
    let address = (0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)) as u32;
    unsafe {
        ptr::write_volatile(0xCF8 as *mut u32, address);
        ptr::read_volatile(0xCFC as *mut u16)
    }
}

pub fn read_config_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = (0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)) as u32;
    unsafe {
        ptr::write_volatile(0xCF8 as *mut u32, address);
        ptr::read_volatile(0xCFC as *mut u32)
    }
}

pub fn write_config_u32(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    let address = (0x80000000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)) as u32;
    unsafe {
        ptr::write_volatile(0xCF8 as *mut u32, address);
        ptr::write_volatile(0xCFC as *mut u32, value);
    }
}

// ===== 修复 #44,#45：PCI桥递归扫描 + 多功能设备处理 =====
pub fn enumerate_pci() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    scan_bus(0, &mut devices, 0);
    devices
}

fn scan_bus(bus: u8, devices: &mut Vec<PciDevice>, depth: u8) {
    if depth > 8 { return; }  // 防止递归过深

    for slot in 0..32 {
        for func in 0..8 {
            let vendor = read_config_u16(bus, slot, func, 0);
            if vendor == 0xFFFF || vendor == 0x0000 {
                if func == 0 { break; }
                continue;
            }

            let header = PciConfigHeader {
                vendor_id: vendor,
                device_id: read_config_u16(bus, slot, func, 2),
                command: read_config_u16(bus, slot, func, 4),
                status: read_config_u16(bus, slot, func, 6),
                revision_id: read_config_u8(bus, slot, func, 8),
                prog_if: read_config_u8(bus, slot, func, 9),
                subclass: read_config_u8(bus, slot, func, 0x0A),
                class: read_config_u8(bus, slot, func, 0x0B),
                cache_line_size: read_config_u8(bus, slot, func, 0x0C),
                latency_timer: read_config_u8(bus, slot, func, 0x0D),
                header_type: read_config_u8(bus, slot, func, 0x0E),
                bist: read_config_u8(bus, slot, func, 0x0F),
                bar0: read_config_u32(bus, slot, func, 0x10),
                bar1: read_config_u32(bus, slot, func, 0x14),
                bar2: read_config_u32(bus, slot, func, 0x18),
                bar3: read_config_u32(bus, slot, func, 0x1C),
                bar4: read_config_u32(bus, slot, func, 0x20),
                bar5: read_config_u32(bus, slot, func, 0x24),
                cardbus_cis_ptr: read_config_u32(bus, slot, func, 0x28),
                subsystem_vendor_id: read_config_u16(bus, slot, func, 0x2C),
                subsystem_id: read_config_u16(bus, slot, func, 0x2E),
                expansion_rom_base_addr: read_config_u32(bus, slot, func, 0x30),
                capabilities_ptr: read_config_u8(bus, slot, func, 0x34),
                reserved0: [0; 3],
                reserved1: 0,
                reserved2: 0,
                interrupt_line: read_config_u8(bus, slot, func, 0x3C),
                interrupt_pin: read_config_u8(bus, slot, func, 0x3D),
                min_grant: read_config_u8(bus, slot, func, 0x3E),
                max_latency: read_config_u8(bus, slot, func, 0x3F),
            };

            devices.push(PciDevice { bus, slot, func, header });

            // ===== 修复 #45：检测PCI桥并递归扫描 =====
            let is_bridge = header.class == 0x06 && header.subclass == 0x04;
            if is_bridge {
                let sec_bus = read_config_u8(bus, slot, func, 0x19);
                scan_bus(sec_bus, devices, depth + 1);
            }

            // ===== 修复 #44：检查多功能设备标志 =====
            if func == 0 {
                let is_multi_function = (header.header_type & 0x80) != 0;
                if !is_multi_function { break; }
            }
        }
    }
}

pub fn find_device(devices: &[PciDevice], class: u8, subclass: u8) -> Option<&PciDevice> {
    devices.iter().find(|d| d.header.class == class && d.header.subclass == subclass)
}
