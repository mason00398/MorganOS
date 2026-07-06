use crate::console::Shell;
use crate::drivers::memory;
use crate::drivers::process;
use crate::drivers::vm;

pub fn run(shell: &mut Shell) {
    // 内存统计
    let mm = memory::get_memory_manager();
    shell.write_str("=== Memory Statistics ===\n");
    shell.write_str(&alloc::format!("Total: {} MB\n", mm.total() / (1024 * 1024)));
    shell.write_str(&alloc::format!("Used: {} MB\n", mm.used() / (1024 * 1024)));
    shell.write_str(&alloc::format!("Free: {} MB\n", mm.free() / (1024 * 1024)));
    shell.write_str(&alloc::format!("Usage: {:.1}%\n\n", mm.usage_percent()));

    // 虚拟内存统计
    shell.write_str("=== Virtual Memory ===\n");
    let vm_mgr = vm::get_vm_manager();
    let (used, total, pct) = vm_mgr.stats();
    shell.write_str(&alloc::format!("Pages allocated: {}/{}\n", used, total));
    shell.write_str(&alloc::format!("Usage: {:.1}%\n", pct));

    // 进程统计
    shell.write_str("\n=== Process Statistics ===\n");
    let sched = process::get_scheduler();
    let procs = sched.list_processes();
    shell.write_str(&alloc::format!("Active processes: {}\n", procs.len()));
    
    for (pid, name, state) in &procs {
        shell.write_str(&alloc::format!("  PID {} ({}) - {:?}\n", pid.0, name, state));
    }
}
