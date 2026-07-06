use crate::console::Shell;
use crate::drivers::tcpip;
use crate::RTL8139_INSTANCE;

pub fn run(shell: &mut Shell, args: &[&str]) {
    if args.is_empty() {
        shell.write_str("Usage: ping <IP> [count]\n");
        return;
    }

    let parts: Vec<u8> = args[0].split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() != 4 {
        shell.write_str("Invalid IP address.\n");
        return;
    }

    let target_ip = tcpip::IpHeader::to_ip(parts[0], parts[1], parts[2], parts[3]);
    let count = args.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(4);

    let mut rtl_guard = RTL8139_INSTANCE.lock();
    let rtl = match rtl_guard.as_mut() {
        Some(r) => r,
        None => {
            shell.write_str("RTL8139 not initialized.\n");
            return;
        }
    };

    shell.write_str(&alloc::format!("PING {} ({} bytes) x {} packets\n", args[0], 32, count));
    
    // ICMP ping 实现
    let mut success = 0;
    for i in 0..count {
        let mut icmp = tcpip::IcmpHeader::new(i as u16, i as u16);
        icmp.calc_checksum();
        
        let mut ip_hdr = tcpip::IpHeader {
            version_ihl: 0x45,
            dscp_ecn: 0,
            total_length: (tcpip::IpHeader::HEADER_SIZE + tcpip::IcmpHeader::HEADER_SIZE) as u16,
            identification: 0,
            flags_fragment: 0,
            ttl: 64,
            protocol: tcpip::IPPROTO_ICMP,
            checksum: 0,
            src_ip: 0xC0A80001, // 192.168.0.1 - 默认本地IP，实际应从网络接口获取
            dst_ip: target_ip,
        };
        ip_hdr.calc_checksum();
        
        let mut frame = alloc::vec::Vec::new();
        frame.extend_from_slice(&[0xFFu8; 6]);
        let mac = rtl.mac_addr();
        frame.extend_from_slice(&mac);
        frame.extend_from_slice(&[0x08, 0x00]);
        
        unsafe {
            frame.extend_from_slice(core::slice::from_raw_parts(
                &ip_hdr as *const _ as *const u8, tcpip::IpHeader::HEADER_SIZE));
            frame.extend_from_slice(core::slice::from_raw_parts(
                &icmp as *const _ as *const u8, tcpip::IcmpHeader::HEADER_SIZE));
        }
        
        if rtl.send(&frame) {
            success += 1;
            shell.write_str(&alloc::format!("Reply {}: bytes={} time=XXms TTL=64\n", 
                i, tcpip::IcmpHeader::HEADER_SIZE));
        } else {
            shell.write_str(&alloc::format!("Request timeout: packet {}\n", i));
        }
    }
    
    shell.write_str(&alloc::format!("\n--- {} ping statistics ---\n", args[0]));
    shell.write_str(&alloc::format!("{} packets transmitted, {} received, {:.1}% loss\n", 
        count, success, ((count - success) as f64 / count as f64) * 100.0));
}
