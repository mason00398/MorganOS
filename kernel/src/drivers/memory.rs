//! 内存管理器

use spin::Mutex;
use linked_list_allocator::Heap;
use core::ptr;

// ===== 修复 #24：统一为16MB =====
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

pub struct MemoryManager {
    heap: Mutex<Heap>,
    allocated: Mutex<usize>,
    total: usize,
}

impl MemoryManager {
    pub const fn new() -> Self {
        Self {
            heap: Mutex::new(Heap::empty()),
            allocated: Mutex::new(0),
            total: HEAP_SIZE,
        }
    }

    pub fn init(&self, start: *mut u8, size: usize) {
        unsafe {
            self.heap.lock().init(start, size);
        }
    }

    // ===== 修复 #23：分配成功后才增加计数 =====
    pub fn alloc(&self, size: usize, align: usize) -> Option<*mut u8> {
        let result = self.heap.lock().alloc(size, align);
        if let Ok(ptr) = result {
            let mut alloc_count = self.allocated.lock();
            *alloc_count += size;
            Some(ptr)
        } else {
            None
        }
    }

    pub fn dealloc(&self, addr: *mut u8, size: usize, align: usize) {
        let result = self.heap.lock().dealloc(addr, size, align);
        if result.is_ok() {
            let mut alloc_count = self.allocated.lock();
            *alloc_count = alloc_count.saturating_sub(size);
        }
    }

    pub fn used(&self) -> usize {
        *self.allocated.lock()
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn free(&self) -> usize {
        self.total - self.used()
    }

    pub fn usage_percent(&self) -> f32 {
        if self.total == 0 { return 0.0; }
        (self.used() as f32 / self.total as f32) * 100.0
    }
}

pub static mut G_MEMORY_MANAGER: Option<MemoryManager> = None;

pub fn get_memory_manager() -> &'static MemoryManager {
    unsafe { G_MEMORY_MANAGER.as_ref().expect("Memory manager not initialized") }
}
