use crate::console::Shell;
use crate::get_pci_devices;
use crate::drivers::{rtc, ahci, rtl8139};

pub fn run(shell: &mut Shell) {
    shell.write_str("=== Hardware Diagnostics ===\n\n");
    let devices = get_pci_devices();
    shell.write_str(&alloc::format!("PCI Devices: {}\n\n", devices.len()));

    // RTC
    let time = rtc::Rtc::get_datetime_string();
    shell.write_str(&alloc::format!("RTC: {}\n", time));

    // AHCI
    if let Some(dev) = crate::pci::find_device(devices, 0x01, 0x06) {
        if let Some(ahci) = ahci::AhciDriver::new(dev) {
            shell.write_str(&alloc::format!("AHCI: {} ports\n", ahci.port_count()));
            for p in 0..ahci.port_count() {
                let connected = ahci.is_connected(p);
                let sig = ahci.get_signature(p);
                shell.write_str(&alloc::format!(
                    "  Port {}: Connected={}, Sig=0x{:08X}\n",
                    p, connected, sig
                ));
            }
        } else {
            shell.write_str("  AHCI: init failed\n");
        }
    } else {
        shell.write_str("  AHCI: not found\n");
    }

    // RTL8139
    if let Some(dev) = crate::pci::find_device(devices, 0x02, 0x00) {
        if let Some(mut rtl) = rtl8139::Rtl8139::new(dev) {
            let mac = rtl.mac_addr();
            shell.write_str(&alloc::format!("RTL8139: MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
            rtl.init();
            let test_data = b"Test packet";
            if rtl.send(test_data) {
                shell.write_str("  Packet sent OK\n");
            } else {
                shell.write_str("  Packet send FAILED\n");
            }
        } else {
            shell.write_str("  RTL8139: init failed\n");
        }
    } else {
        shell.write_str("  RTL8139: not found\n");
    }

    // Memory
    use crate::drivers::memory;
    let mem = memory::MemInfo::get();
    shell.write_str(&alloc::format!("\nMemory: {}KB used / {}KB total ({:.1}%)\n",
        mem.used / 1024, mem.total / 1024, mem.usage_pct));

    shell.write_str("\nDiagnostics complete.\n");
}
