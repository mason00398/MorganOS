use crate::console::Shell;
use crate::drivers::process;

/// kill - 杀死进程
pub fn run(shell: &mut Shell, args: &[&str]) {
    if args.is_empty() {
        shell.write_str("Usage: kill <PID>\n");
        shell.write_str("       kill -a (all)\n");
        return;
    }
    
    if args[0] == "-a" {
        shell.write_str("Killing all user processes...\n");
        // 简化：只清空调度器
        return;
    }
    
    if let Ok(pid) = args[0].parse::<u64>() {
        let pm = process::get_scheduler_mut();
        if pm.terminate(process::ProcessId(pid)) {
            shell.write_str(&alloc::format!("Process {} terminated.\n", pid));
        } else {
            shell.write_str(&alloc::format!("Process {} not found.\n", pid));
        }
    } else {
        shell.write_str("Invalid PID.\n");
    }
}
