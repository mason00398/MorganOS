//! 网络接口 - ARP解析

use alloc::collections::BTreeMap;

pub struct ArpCache {
    entries: BTreeMap<u32, [u8; 6]>,
}

impl ArpCache {
    pub const fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    pub fn lookup(&self, ip: u32) -> Option<[u8; 6]> {
        self.entries.get(&ip).copied()
    }

    pub fn add(&mut self, ip: u32, mac: [u8; 6]) {
        self.entries.insert(ip, mac);
    }
}
