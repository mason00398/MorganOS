use crate::console::Shell;
use crate::get_pci_devices;
use crate::drivers::rtl8139;

/// ifconfig - 显示网络接口信息
pub fn run(shell: &mut Shell) {
    let devices = get_pci_devices();
    if let Some(dev) = crate::pci::find_device(devices, 0x02, 0x00) {
        match rtl8139::Rtl8139::new(dev) {
            Some(rtl) => {
                let mac = rtl.mac_addr();
                shell.write_str("eth0: flags=UP,BROADCAST  mtu 1500\n");
                shell.write_str(&alloc::format!("        ether {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
                shell.write_str("        inet 0.0.0.0  netmask 0.0.0.0  (DHCP not implemented)\n");
            }
            None => shell.write_str("RTL8139 not found.\n"),
        }
    } else {
        shell.write_str("No network interface found.\n");
    }
}
