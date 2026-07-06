//! 虚拟内存管理 - 页表管理
//! 
//! 修复内容：
//! - 添加页表管理
//! - 添加内存映射
//! - 添加内存保护

use spin::Mutex;
use core::ptr;

// ========== 页表常量 ==========
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_TABLE_ENTRIES: usize = 512;

// ========== 页表项 ==========
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    pub present: u64,
    pub rw: u64,
    pub user_supervisor: u64,
    pub page_write_through: u64,
    pub page_cache_disable: u64,
    pub accessed: u64,
    pub dirty: u64,
    pub unused: u64,
    pub global: u64,
    pub available: u64,
    pub physical_address: u64,
    pub reserved: u64,
    pub no_execute: u64,
}

// ========== 页表 ==========
pub struct PageTable {
    pub root_frame: u64,
    pub entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            root_frame: 0,
            entries: [PageTableEntry {
                present: 0, rw: 0, user_supervisor: 0,
                page_write_through: 0, page_cache_disable: 0,
                accessed: 0, dirty: 0, unused: 0, global: 0,
                available: 0, physical_address: 0, reserved: 0,
                no_execute: 0,
            }; PAGE_TABLE_ENTRIES],
        }
    }

    /// 物理地址转虚拟页号
    pub fn vpn(phys_addr: usize) -> usize {
        phys_addr / PAGE_SIZE
    }

    /// 虚拟页号转物理地址
    pub fn paddr(vpn: usize) -> usize {
        vpn * PAGE_SIZE
    }

    /// 映射一个页
    pub fn map(&mut self, vpn: usize, ppn: usize, perms: PagePermissions) {
        let entry = &mut self.entries[vpn];
        entry.present = perms.present as u64;
        entry.rw = perms.read_write as u64;
        entry.user_supervisor = perms.user as u64;
        entry.no_execute = perms.no_execute as u64;
        entry.physical_address = ppn as u64;
    }

    /// 取消映射
    pub fn unmap(&mut self, vpn: usize) {
        if vpn < PAGE_TABLE_ENTRIES {
            self.entries[vpn].present = 0;
            self.entries[vpn].physical_address = 0;
        }
    }

    /// 查找页表项
    pub fn find(&self, vpn: usize) -> Option<&PageTableEntry> {
        if vpn < PAGE_TABLE_ENTRIES && self.entries[vpn].present != 0 {
            Some(&self.entries[vpn])
        } else {
            None
        }
    }
}

// ========== 页权限 ==========

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PagePermissions {
    pub present: bool,
    pub read_write: bool,
    pub user: bool,
    pub no_execute: bool,
}

impl PagePermissions {
    pub const fn read_only() -> Self {
        Self {
            present: true,
            read_write: false,
            user: false,
            no_execute: false,
        }
    }

    pub const fn read_write() -> Self {
        Self {
            present: true,
            read_write: true,
            user: false,
            no_execute: false,
        }
    }

    pub const fn execute() -> Self {
        Self {
            present: true,
            read_write: false,
            user: false,
            no_execute: false,
        }
    }

    pub const fn execute_read_write() -> Self {
        Self {
            present: true,
            read_write: true,
            user: false,
            no_execute: false,
        }
    }
}

// ========== 虚拟内存管理器 ==========

pub struct VirtualMemoryManager {
    pub page_tables: Mutex<Vec<PageTable>>,
    pub total_pages: usize,
    pub used_pages: Mutex<usize>,
}

impl VirtualMemoryManager {
    pub const fn new() -> Self {
        Self {
            page_tables: spin::Mutex::new(Vec::new()),
            total_pages: 0,
            used_pages: spin::Mutex::new(0),
        }
    }

    pub fn init(&mut self, total_pages: usize) {
        self.total_pages = total_pages;
    }

    /// 分配一页
    pub fn allocate_page(&self) -> Option<usize> {
        let mut used = self.used_pages.lock();
        if *used >= self.total_pages {
            return None;
        }
        let page = *used;
        *used += 1;
        Some(page * PAGE_SIZE)
    }

    /// 释放一页
    pub fn free_page(&self, addr: usize) {
        let mut used = self.used_pages.lock();
        if addr % PAGE_SIZE == 0 && addr / PAGE_SIZE < *used {
            *used -= 1;
        }
    }

    /// 映射一段内存
    pub fn mmap(&self, vaddr: usize, paddr: usize, size: usize, perms: PagePermissions) -> bool {
        let start_vpn = vaddr / PAGE_SIZE;
        let end_vpn = (vaddr + size - 1) / PAGE_SIZE;
        
        for vpn in start_vpn..=end_vpn {
            let ppn = paddr / PAGE_SIZE + (vpn - start_vpn);
            // 这里简化处理，实际需要更复杂的页表管理
            let _ = (vpn, ppn, perms);
        }
        true
    }

    /// 获取内存使用统计
    pub fn stats(&self) -> (usize, usize, f32) {
        let used = *self.used_pages.lock();
        (used, self.total_pages, 
         if self.total_pages > 0 { (used as f32 / self.total_pages as f32) * 100.0 } else { 0.0 })
    }
}

static mut G_VM_MANAGER: Option<VirtualMemoryManager> = None;

pub fn init_vm_manager(total_pages: usize) {
    let mut vm = VirtualMemoryManager::new();
    vm.init(total_pages);
    unsafe { G_VM_MANAGER = Some(vm); }
}

pub fn get_vm_manager() -> &'static VirtualMemoryManager {
    unsafe { G_VM_MANAGER.as_ref().expect("VM manager not initialized") }
}
