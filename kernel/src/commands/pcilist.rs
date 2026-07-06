use crate::console::Shell;
use crate::get_pci_devices;

pub fn run(shell: &mut Shell) {
    let devices = get_pci_devices();
    if devices.is_empty() {
        shell.write_str("No PCI devices found.\n");
        return;
    }
    shell.write_str("Bus Slot Func Vendor Device Class Subclass\n");
    for d in devices {
        shell.write_str(&alloc::format!("{:02X}  {:02X}   {:02X}  {:04X}   {:04X}   {:02X}    {:02X}\n",
            d.bus, d.slot, d.func,
            d.header.vendor_id, d.header.device_id,
            d.header.class, d.header.subclass));
    }
    shell.write_str(&alloc::format!("Total: {}\n", devices.len()));
}
