#![no_std]
#![no_main]

extern crate alloc;

use uefi::prelude::*;
use linked_list_allocator::LockedHeap;
use alloc::vec::Vec;
use spin::Mutex;

mod console;
mod commands;
mod pci;
mod drivers;
mod panic;

pub use drivers::tcpip;
pub use drivers::vm;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// ===== 全局驱动实例（修复 #20,#21,#22） =====
static AHCI_INSTANCE: Mutex<Option<drivers::ahci::AhciManager>> = Mutex::new(None);
static RTL8139_INSTANCE: Mutex<Option<drivers::rtl8139::Rtl8139>> = Mutex::new(None);
static FAT32_INSTANCE: Mutex<Option<drivers::fat32::Fat32Fs>> = Mutex::new(None);
static PCI_DEVICES: Mutex<Option<Vec<pci::PciDevice>>> = Mutex::new(None);

#[entry]
fn efi_main(_handle: Handle, mut st: SystemTable<Boot>) -> Status {
    if let Err(_) = uefi_services::init(&mut st) {
        return Status::LOAD_ERROR;
    }

    uefi_services::println!("RunST X Kernel v0.4");
    uefi_services::println!("Build: {}", env!("BUILD_DATE"));
    uefi_services::println!("Git: {}", env!("GIT_HASH"));

    // ===== 修复 #17：堆从4MB改为16MB =====
    let heap_size = 16 * 1024 * 1024;
    let heap_pages = (heap_size + 0xFFF) / 0x1000;
    let heap_start = match st.boot_services().allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        uefi::table::boot::MemoryType::LOADER_DATA,
        heap_pages,
    ) {
        Ok(addr) => addr as *mut u8,
        Err(_) => {
            uefi_services::println!("Failed to allocate heap");
            return Status::OUT_OF_RESOURCES;
        }
    };
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
    uefi_services::println!("Heap: 16 MB at 0x{:x}", heap_start as usize);

    // ===== 修复 #18：正确初始化内存管理器 =====
    unsafe {
        let mm = drivers::memory::MemoryManager::new();
        mm.init(heap_start, heap_size);
        drivers::memory::G_MEMORY_MANAGER = Some(mm);
    }

    // ===== 初始化调度器 =====
    drivers::process::init_scheduler();
    uefi_services::println!("Scheduler: initialized");

    // ===== 初始化虚拟内存管理器 =====
    let total_pages = (heap_size / vm::PAGE_SIZE) as usize;
    vm::init_vm_manager(total_pages);
    uefi_services::println!("Virtual Memory: {} pages ({} MB)", total_pages, heap_size / (1024 * 1024));

    // ===== PCI枚举 =====
    let devices = pci::enumerate_pci();
    uefi_services::println!("PCI: {} devices found", devices.len());
    *PCI_DEVICES.lock() = Some(devices);

    // ===== 修复 #20：保存AHCI实例 =====
    if let Some(dev) = pci::find_device(&get_pci_devices(), 0x01, 0x06) {
        if let Some(ahci) = drivers::ahci::AhciManager::new(dev) {
            uefi_services::println!("AHCI: {} ports", ahci.port_count());
            *AHCI_INSTANCE.lock() = Some(ahci);
        }
    }

    // ===== 修复 #21：保存RTL8139实例 =====
    if let Some(dev) = pci::find_device(&get_pci_devices(), 0x02, 0x00) {
        if let Some(mut rtl) = drivers::rtl8139::Rtl8139::new(dev) {
            let mac = rtl.mac_addr();
            uefi_services::println!("RTL8139: MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
            rtl.init();
            *RTL8139_INSTANCE.lock() = Some(rtl);
        }
    }

    // ===== 修复 #19：正确退出Boot Services =====
    uefi_services::println!("Exiting Boot Services...");
    let _memory_map = unsafe {
        st.exit_boot_services(Some(uefi::table::boot::MemoryType::LOADER_DATA))
    };

    // ===== VGA Shell =====
    let mut shell = console::Shell::new();
    shell.write_str("RunST X v0.4 ready\nType 'help' for commands\n\n");

    drivers::keyboard::init();

    loop {
        shell.print_prompt();
        if let Some(line) = shell.read_line() {
            commands::dispatch(&mut shell, line);
        }
    }
}

pub fn get_pci_devices() -> &'static [pci::PciDevice] {
    PCI_DEVICES.lock().as_ref().map(|v| v.as_slice()).unwrap_or(&[])
}
