use crate::console::Shell;
use crate::drivers::tcpip;
use crate::RTL8139_INSTANCE;

pub fn run(shell: &mut Shell, args: &[&str]) {
    if args.is_empty() {
        shell.write_str("Usage: net <ping|status|socket|dns> [args...]\n");
        return;
    }

    match args[0] {
        "status" => {
            let mut rtl_guard = RTL8139_INSTANCE.lock();
            let rtl = match rtl_guard.as_mut() {
                Some(r) => r,
                None => {
                    shell.write_str("RTL8139 not initialized.\n");
                    return;
                }
            };
            let mac = rtl.mac_addr();
            shell.write_str(&alloc::format!("RTL8139: MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]));
            shell.write_str("Network interface ready.\n");
        }
        "ping" => {
            if args.len() < 2 {
                shell.write_str("Usage: net ping <ip_address>\n");
                return;
            }
            let ip_str = args[1];
            let parts: Vec<&str> = ip_str.split('.').collect();
            if parts.len() != 4 {
                shell.write_str("Invalid IP format. Use: a.b.c.d\n");
                return;
            }
            let ip = tcpip::IpHeader::to_ip(
                parts[0].parse().unwrap_or(0),
                parts[1].parse().unwrap_or(0),
                parts[2].parse().unwrap_or(0),
                parts[3].parse().unwrap_or(0),
            );
            
            let mut rtl_guard = RTL8139_INSTANCE.lock();
            let rtl = match rtl_guard.as_mut() {
                Some(r) => r,
                None => {
                    shell.write_str("RTL8139 not initialized.\n");
                    return;
                }
            };
            shell.write_str(&alloc::format!("PING {} ...\n", ip_str));
            // 简单的 ping 实现
            shell.write_str("Ping not fully implemented yet.\n");
        }
        "socket" => {
            shell.write_str("Socket API:\n");
            shell.write_str("  - TCP sockets: connect(), send(), recv(), close()\n");
            shell.write_str("  - UDP sockets: send(), recv()\n");
            shell.write_str("  - DNS lookup: dns_lookup(domain)\n");
        }
        "dns" => {
            if args.len() < 2 {
                shell.write_str("Usage: net dns <domain>\n");
                return;
            }
            let domain = args[1];
            shell.write_str(&alloc::format!("DNS lookup for: {}\n", domain));
            shell.write_str("DNS resolution: cached (limited)\n");
        }
        _ => {
            shell.write_str("Unknown net command. Use: status, ping, socket, dns\n");
        }
    }
}
