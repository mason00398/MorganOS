//! FAT32文件系统 - 基于fat32 crate

use alloc::vec::Vec;
use alloc::string::String;

// ===== 修复 #13：BlockDevice trait =====
pub trait BlockDevice: Send + Sync {
    fn read_sector(&mut self, sector: u32, buf: &mut [u8; 512]) -> bool;
    fn write_sector(&mut self, sector: u32, buf: &[u8; 512]) -> bool;
}

pub struct Fat32Fs {
    disk: Option<Box<dyn BlockDevice>>,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    sectors_per_fat: u32,
    root_cluster: u32,
    data_start_sector: u32,
}

impl Fat32Fs {
    // ===== 修复 #14：接收disk参数 =====
    pub fn new(mut disk: Box<dyn BlockDevice>) -> Option<Self> {
        let mut bpb_buf = [0u8; 512];
        if !disk.read_sector(0, &mut bpb_buf) {
            return None;
        }

        // 解析BPB
        let bytes_per_sector = u16::from_le_bytes([bpb_buf[11], bpb_buf[12]]);
        let sectors_per_cluster = bpb_buf[13];
        let reserved_sectors = u16::from_le_bytes([bpb_buf[14], bpb_buf[15]]);
        let fat_count = bpb_buf[16];
        let sectors_per_fat = u32::from_le_bytes([bpb_buf[36], bpb_buf[37], bpb_buf[38], bpb_buf[39]]);
        let root_cluster = u32::from_le_bytes([bpb_buf[44], bpb_buf[45], bpb_buf[46], bpb_buf[47]]);

        // 检查FAT32签名
        if bpb_buf[510] != 0x55 || bpb_buf[511] != 0xAA {
            return None;
        }

        let data_start_sector = reserved_sectors as u32 + (fat_count as u32) * sectors_per_fat;

        Some(Self {
            disk: Some(disk),
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            fat_count,
            sectors_per_fat,
            root_cluster,
            data_start_sector,
        })
    }

    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        self.data_start_sector + (cluster - 2) * self.sectors_per_cluster as u32
    }

    // ===== 修复 #15：使用 &mut self 避免clone =====
    fn read_fat_entry(&mut self, cluster: u32) -> u32 {
        let disk = match self.disk.as_mut() {
            Some(d) => d,
            None => return 0,
        };
        let fat_offset = cluster * 4;
        let sector = self.reserved_sectors as u32 + fat_offset / 512;
        let offset = (fat_offset % 512) as usize;

        let mut buf = [0u8; 512];
        if !disk.read_sector(sector, &mut buf) {
            return 0;
        }
        let val = u32::from_le_bytes([buf[offset], buf[offset+1], buf[offset+2], buf[offset+3]]);
        val & 0x0FFFFFFF
    }

    fn read_cluster_chain(&mut self, start_cluster: u32) -> Vec<u32> {
        let mut clusters = Vec::new();
        let mut current = start_cluster;
        while current > 0 && current < 0x0FFFFFF8 {
            clusters.push(current);
            current = self.read_fat_entry(current);
        }
        clusters
    }

    // ===== 修复 #16：使用 &mut self 避免take =====
    pub fn read_file(&mut self, cluster: u32, size: u32, buf: &mut [u8]) -> bool {
        let disk = match self.disk.as_mut() {
            Some(d) => d,
            None => return false,
        };
        let clusters = self.read_cluster_chain(cluster);

        let mut bytes_read = 0u32;
        for cl in &clusters {
            if bytes_read >= size { break; }

            let sector = self.cluster_to_sector(*cl);
            let bytes_in_cluster = self.sectors_per_cluster as u32 * 512;
            let to_read = core::cmp::min(bytes_in_cluster, size - bytes_read);

            let sectors_needed = ((to_read + 511) / 512) as usize;
            let mut sector_buf = [0u8; 512];
            for s in 0..sectors_needed {
                if !disk.read_sector(sector + s as u32, &mut sector_buf) {
                    return false;
                }
                let start = (s * 512) as usize;
                let end = core::cmp::min(start + 512, to_read as usize);
                let dst_start = bytes_read as usize;
                let dst_end = core::cmp::min(dst_start + (end - start), buf.len());
                if dst_end > dst_start {
                    let copy_len = core::cmp::min(end - start, dst_end - dst_start);
                    let src_start = start;
                    core::ptr::copy_nonoverlapping(
                        sector_buf.as_ptr().add(src_start),
                        buf.as_mut_ptr().add(dst_start),
                        copy_len,
                    );
                }
                bytes_read += (end - start) as u32;
            }
        }
        bytes_read == size
    }

    pub fn read_directory<F: FnMut(&str, u32, u32)>(&mut self, cluster: u32, mut callback: F) {
        let disk = match self.disk.as_mut() {
            Some(d) => d,
            None => return,
        };
        let clusters = self.read_cluster_chain(cluster);

        for cl in &clusters {
            let sector = self.cluster_to_sector(*cl);
            let entries_per_sector = 512 / 32;

            let mut sector_buf = [0u8; 512];
            for s in 0..self.sectors_per_cluster {
                if !disk.read_sector(sector + s as u32, &mut sector_buf) { break; }

                for entry_idx in 0..entries_per_sector {
                    let offset = entry_idx * 32;
                    if offset + 32 > 512 { break; }

                    let name = &sector_buf[offset..offset+11];
                    if name[0] == 0x00 { break; }
                    if name[0] == 0xE5 { continue; }

                    let attr = sector_buf[offset + 11];
                    let cluster_low = u16::from_le_bytes([sector_buf[offset+26], sector_buf[offset+27]]);
                    let cluster_high = u16::from_le_bytes([sector_buf[offset+20], sector_buf[offset+21]]);
                    let cluster = ((cluster_high as u32) << 16) | (cluster_low as u32);
                    let size = u32::from_le_bytes([
                        sector_buf[offset+28], sector_buf[offset+29],
                        sector_buf[offset+30], sector_buf[offset+31],
                    ]);

                    let mut name_str = String::new();
                    for i in 0..8 {
                        if name[i] == 0x20 { break; }
                        name_str.push(name[i] as char);
                    }
                    if attr & 0x10 == 0 {
                        name_str.push('.');
                        for i in 8..11 {
                            if name[i] == 0x20 { break; }
                            name_str.push(name[i] as char);
                        }
                    }
                    callback(&name_str, cluster, size);
                }
            }
        }
    }
}
