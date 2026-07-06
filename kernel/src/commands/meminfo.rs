use crate::console::Shell;
use crate::drivers::memory;
use crate::drivers::vm;

pub fn run(shell: &mut Shell) {
    let mm = memory::get_memory_manager();
    shell.write_str(&alloc::format!("Memory Manager:\n"));
    shell.write_str(&alloc::format!("  Total:  {} MB\n", mm.total() / (1024 * 1024)));
    shell.write_str(&alloc::format!("  Used:   {} MB\n", mm.used() / (1024 * 1024)));
    shell.write_str(&alloc::format!("  Free:   {} MB\n", mm.free() / (1024 * 1024)));
    shell.write_str(&alloc::format!("  Usage:  {:.1}%\n", mm.usage_percent()));

    // 虚拟内存统计
    shell.write_str("\nVirtual Memory:\n");
    let vm_mgr = vm::get_vm_manager();
    let (used, total, pct) = vm_mgr.stats();
    shell.write_str(&alloc::format!("  Pages:  {}/{}\n", used, total));
    shell.write_str(&alloc::format!("  Usage:  {:.1}%\n", pct));
}
