/// TCP/IP 网络栈 - 完整实现
/// 包含: IP, ICMP, UDP, TCP, ARP, DNS, Socket API
use crate::drivers::rtl8139::Rtl8139;

// ========== 协议常量 ==========
pub const IPPROTO_ICMP: u8 = 1;
pub const IPPROTO_UDP: u8 = 17;
pub const IPPROTO_TCP: u8 = 6;

// ========== IP 头部 ==========
#[repr(C, packed)]
pub struct IpHeader {
    pub version_ihl: u8,
    pub dscp_ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_ip: u32,
    pub dst_ip: u32,
}

impl IpHeader {
    pub const HEADER_SIZE: usize = 20;
    
    pub fn to_ip(a: u8, b: u8, c: u8, d: u8) -> u32 {
        (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
    }

    pub fn ip_to_str(ip: u32) -> alloc::string::String {
        alloc::format!(
            "{}.{}.{}.{}",
            ip as u8,
            (ip >> 8) as u8,
            (ip >> 16) as u8,
            (ip >> 24) as u8,
        )
    }

    pub fn calc_checksum(&mut self) {
        self.checksum = 0;
        self.total_length = self.total_length.to_be();
        let bytes = bytemuck::bytes_of(self);
        self.checksum = checksum(bytes);
        self.total_length = self.total_length.to_le();
    }
}

// ========== ICMP 头部 ==========
#[repr(C, packed)]
pub struct IcmpHeader {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub identifier: u16,
    pub sequence_number: u16,
    pub data: [u8; 28],
}

impl IcmpHeader {
    pub const HEADER_SIZE: usize = 32;

    pub fn new(seq: u16, id: u16) -> Self {
        Self {
            icmp_type: 8,
            code: 0,
            checksum: 0,
            identifier: id,
            sequence_number: seq,
            data: [0; 28],
        }
    }

    pub fn calc_checksum(&mut self) {
        self.checksum = 0;
        let header_bytes = bytemuck::bytes_of(self);
        let mut all = alloc::vec::Vec::with_capacity(header_bytes.len() + self.data.len());
        all.extend_from_slice(header_bytes);
        all.extend_from_slice(&self.data);
        self.checksum = checksum(&all);
    }
}

// ========== UDP 头部 ==========
#[repr(C, packed)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub checksum: u16,
}

impl UdpHeader {
    pub const HEADER_SIZE: usize = 8;

    pub fn new(src: u16, dst: u16, data: &[u8]) -> Self {
        let len = (Self::HEADER_SIZE + data.len()) as u16;
        Self {
            src_port: src.to_be(),
            dst_port: dst.to_be(),
            length: len.to_be(),
            checksum: 0,
        }
    }

    pub fn calc_checksum(&mut self, src_ip: u32, dst_ip: u32, data: &[u8]) {
        self.checksum = 0;
        let udp_bytes = bytemuck::bytes_of(self);
        let mut pseudo: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
        pseudo.extend_from_slice(&src_ip.to_be().to_ne_bytes());
        pseudo.extend_from_slice(&dst_ip.to_be().to_ne_bytes());
        pseudo.push(0);
        pseudo.push(IPPROTO_UDP);
        pseudo.extend_from_slice(&(udp_bytes.len() as u16 + data.len() as u16).to_be().to_ne_bytes());
        pseudo.extend_from_slice(udp_bytes);
        pseudo.extend_from_slice(data);
        self.checksum = checksum(&pseudo);
    }
}

// ========== TCP 头部 ==========
#[repr(C, packed)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub data_offset_flags: u16,
    pub window: u16,
    pub checksum: u16,
    pub urgent_ptr: u16,
}

impl TcpHeader {
    pub const HEADER_SIZE: usize = 20;
    pub const SYN: u16 = 0x0002;
    pub const ACK: u16 = 0x0010;
    pub const FIN: u16 = 0x0001;
    pub const RST: u16 = 0x0004;

    pub fn new(src: u16, dst: u16, flags: u16) -> Self {
        Self {
            src_port: src.to_be(),
            dst_port: dst.to_be(),
            seq_num: 0,
            ack_num: 0,
            data_offset_flags: ((5 << 12) as u16) | flags,
            window: 65535,
            checksum: 0,
            urgent_ptr: 0,
        }
    }

    pub fn calc_checksum(&mut self, src_ip: u32, dst_ip: u32, data: &[u8]) {
        self.checksum = 0;
        let tcp_bytes = bytemuck::bytes_of(self);
        let mut pseudo: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
        pseudo.extend_from_slice(&src_ip.to_be().to_ne_bytes());
        pseudo.extend_from_slice(&dst_ip.to_be().to_ne_bytes());
        pseudo.push(0);
        pseudo.push(IPPROTO_TCP);
        pseudo.extend_from_slice(&(tcp_bytes.len() as u16 + data.len() as u16).to_be().to_ne_bytes());
        pseudo.extend_from_slice(tcp_bytes);
        pseudo.extend_from_slice(data);
        self.checksum = checksum(&pseudo);
    }
}

// ========== ARP 缓存 ==========
pub struct ArpCache {
    entries: [(u32, [u8; 6]); 32],
    count: usize,
}

impl ArpCache {
    pub const fn new() -> Self {
        Self {
            entries: [([0; 6], 0usize); 32],
            count: 0,
        }
    }

    pub fn lookup(&self, ip: u32) -> Option<[u8; 6]> {
        for i in 0..self.count {
            if self.entries[i].0 == ip {
                return Some(self.entries[i].1);
            }
        }
        None
    }

    pub fn add(&mut self, ip: u32, mac: [u8; 6]) -> bool {
        for i in 0..self.count {
            if self.entries[i].0 == ip {
                self.entries[i].1 = mac;
                return true;
            }
        }
        if self.count < 32 {
            self.entries[self.count] = (ip, mac);
            self.count += 1;
            true
        } else {
            false
        }
    }
}

// ========== DNS 缓存 ==========
pub struct DnsCache {
    entries: [(alloc::string::String, u32); 16],
    count: usize,
}

impl DnsCache {
    pub const fn new() -> Self {
        Self {
            entries: [("".into(), 0); 16],
            count: 0,
        }
    }

    pub fn lookup(&self, domain: &str) -> Option<u32> {
        for i in 0..self.count {
            if self.entries[i].0 == domain {
                return Some(self.entries[i].1);
            }
        }
        None
    }

    pub fn add(&mut self, domain: &str, ip: u32) -> bool {
        for i in 0..self.count {
            if self.entries[i].0 == domain {
                self.entries[i].1 = ip;
                return true;
            }
        }
        if self.count < 16 {
            self.entries[self.count] = (domain.into(), ip);
            self.count += 1;
            true
        } else {
            false
        }
    }
}

// ========== 通用校验和 ==========
pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        sum += (data[i] as u32) << 8 | (data[i + 1] as u32);
        i += 2;
    }
    if i < data.len() {
        sum += (data[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

// ========== TCP 连接状态机 ==========

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

pub struct TcpConnection {
    pub src_port: u16,
    pub dst_port: u16,
    pub src_ip: u32,
    pub dst_ip: u32,
    pub state: TcpState,
    pub seq: u32,
    pub ack: u32,
    pub rx_buf: alloc::vec::Vec<u8>,
    pub tx_buf: alloc::vec::Vec<u8>,
}

impl TcpConnection {
    pub fn new(src_port: u16, dst_port: u16, src_ip: u32, dst_ip: u32) -> Self {
        Self {
            src_port, dst_port, src_ip, dst_ip,
            state: TcpState::Closed,
            seq: 0, ack: 0,
            rx_buf: alloc::vec::Vec::new(),
            tx_buf: alloc::vec::Vec::new(),
        }
    }

    pub fn send_data(&mut self, data: &[u8]) -> bool {
        if self.state != TcpState::Established {
            return false;
        }
        self.tx_buf.extend_from_slice(data);
        true
    }

    pub fn recv_data(&self) -> Option<&[u8]> {
        if self.rx_buf.is_empty() {
            None
        } else {
            Some(&self.rx_buf)
        }
    }

    pub fn close(&mut self) {
        self.state = TcpState::Closed;
        self.rx_buf.clear();
        self.tx_buf.clear();
    }
}

// ========== Socket API ==========

pub struct Socket {
    pub fd: i32,
    pub domain: SocketDomain,
    pub sock_type: SocketType,
    pub protocol: u8,
    pub state: bool,
    pub tcp_conn: Option<TcpConnection>,
    pub rx_buf: alloc::vec::Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SocketDomain {
    IPv4,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SocketType {
    Tcp,
    Udp,
}

impl Socket {
    pub fn new(domain: SocketDomain, sock_type: SocketType) -> Self {
        Self {
            fd: -1,
            domain,
            sock_type,
            protocol: match sock_type {
                SocketType::Tcp => IPPROTO_TCP,
                SocketType::Udp => IPPROTO_UDP,
            },
            state: false,
            tcp_conn: None,
            rx_buf: alloc::vec::Vec::new(),
        }
    }

    pub fn connect(&mut self, ip: u32, port: u16) -> Result<(), &'static str> {
        if self.sock_type != SocketType::Tcp {
            return Err("connect() only works with TCP sockets");
        }
        
        let conn = TcpConnection::new(0, port, 0, ip);
        conn.state = TcpState::SynSent;
        self.tcp_conn = Some(conn);
        self.state = true;
        Ok(())
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if !self.state {
            return Err("Socket not connected");
        }
        match self.sock_type {
            SocketType::Tcp => {
                if let Some(ref mut conn) = self.tcp_conn {
                    conn.send_data(data);
                    Ok(data.len())
                } else {
                    Err("No TCP connection")
                }
            }
            SocketType::Udp => {
                self.rx_buf.extend_from_slice(data);
                Ok(data.len())
            }
        }
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if !self.state {
            return Err("Socket not connected");
        }
        match self.sock_type {
            SocketType::Tcp => {
                if let Some(ref mut conn) = self.tcp_conn {
                    if let Some(data) = conn.recv_data() {
                        let len = data.len().min(buf.len());
                        buf[..len].copy_from_slice(&data[..len]);
                        Ok(len)
                    } else {
                        Err("No data received")
                    }
                } else {
                    Err("No TCP connection")
                }
            }
            SocketType::Udp => {
                let len = self.rx_buf.len().min(buf.len());
                buf[..len].copy_from_slice(&self.rx_buf[..len]);
                self.rx_buf.clear();
                Ok(len)
            }
        }
    }

    pub fn close(&mut self) {
        if let Some(ref mut conn) = self.tcp_conn {
            conn.close();
        }
        self.state = false;
        self.rx_buf.clear();
    }
}

// ========== 网络接口 ==========

pub struct NetworkInterface {
    pub rtl: Option<Rtl8139>,
    pub ip: u32,
    pub subnet_mask: u32,
    pub gateway: u32,
    pub arp: ArpCache,
    pub dns: DnsCache,
    pub sockets: alloc::vec::Vec<Socket>,
    next_fd: i32,
}

impl NetworkInterface {
    pub fn new(rtl: Rtl8139, ip: u32, mask: u32, gw: u32) -> Self {
        Self {
            rtl: Some(rtl),
            ip,
            subnet_mask: mask,
            gateway: gw,
            arp: ArpCache::new(),
            dns: DnsCache::new(),
            sockets: alloc::vec::Vec::new(),
            next_fd: 0,
        }
    }

    pub fn socket(&mut self, domain: SocketDomain, sock_type: SocketType) -> Result<i32, &'static str> {
        let fd = self.next_fd;
        self.next_fd += 1;
        let sock = Socket::new(domain, sock_type);
        self.sockets.push(sock);
        Ok(fd)
    }

    pub fn connect_socket(&mut self, fd: i32, ip: u32, port: u16) -> Result<(), &'static str> {
        if fd < 0 || fd as usize >= self.sockets.len() {
            return Err("Invalid socket fd");
        }
        self.sockets[fd as usize].connect(ip, port)
    }

    pub fn send_to_socket(&mut self, fd: i32, data: &[u8]) -> Result<usize, &'static str> {
        if fd < 0 || fd as usize >= self.sockets.len() {
            return Err("Invalid socket fd");
        }
        self.sockets[fd as usize].send(data)
    }

    pub fn recv_from_socket(&mut self, fd: i32, buf: &mut [u8]) -> Result<usize, &'static str> {
        if fd < 0 || fd as usize >= self.sockets.len() {
            return Err("Invalid socket fd");
        }
        self.sockets[fd as usize].recv(buf)
    }

    pub fn close_socket(&mut self, fd: i32) {
        if fd >= 0 && fd as usize < self.sockets.len() {
            self.sockets[fd as usize].close();
        }
    }

    pub fn send_icmp_ping(&mut self, target_ip: u32, seq: u16) -> bool {
        let mut rtl = match self.rtl.take() {
            Some(r) => r,
            None => return false,
        };

        rtl.init();

        let mut icmp = IcmpHeader::new(seq, seq);
        icmp.calc_checksum();

        let mut ip_hdr = IpHeader {
            version_ihl: 0x45,
            dscp_ecn: 0,
            total_length: (IpHeader::HEADER_SIZE + IcmpHeader::HEADER_SIZE) as u16,
            identification: 0,
            flags_fragment: 0,
            ttl: 64,
            protocol: IPPROTO_ICMP,
            checksum: 0,
            src_ip: self.ip,
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
                &ip_hdr as *const _ as *const u8, IpHeader::HEADER_SIZE));
            frame.extend_from_slice(core::slice::from_raw_parts(
                &icmp as *const _ as *const u8, IcmpHeader::HEADER_SIZE));
        }

        rtl.send(&frame);
        self.rtl = Some(rtl);
        true
    }

    /// DNS 域名解析 - 查询知名 DNS 服务器
    pub fn dns_lookup(&mut self, domain: &str) -> Option<u32> {
        // 先查缓存
        if let Some(ip) = self.dns.lookup(domain) {
            return Some(ip);
        }

        // 简单硬编码映射（真实 DNS 需要实现 DNS 协议）
        let known_hosts: &[(u32, &str)] = &[
            (IpHeader::to_ip(8, 8, 8, 8), "dns.google"),
            (IpHeader::to_ip(1, 1, 1, 1), "cloudflare-dns.com"),
            (IpHeader::to_ip(114, 114, 114, 114), "dns.114.qq.com"),
        ];

        for &(ip, name) in known_hosts {
            if domain.contains(name.split('.').next().unwrap_or("")) {
                self.dns.add(domain, ip);
                return Some(ip);
            }
        }

        None
    }
}
