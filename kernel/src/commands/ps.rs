use crate::console::Shell;

/// ps - 列出进程（使用新的 Scheduler）
pub fn run(shell: &mut Shell) {
    let pm = crate::drivers::process::get_scheduler();
    let procs = pm.list_processes();
    if procs.is_empty() {
        shell.write_str("No processes running.\n");
        return;
    }
    shell.write_str("  PID  NAME              STATE       PRI  CHILDREN\n");
    shell.write_str("  ---  ----------------  ----------  ---  --------\n");
    for (id, name, state) in &procs {
        let state_str = match state {
            crate::drivers::process::ProcessState::Ready => "READY",
            crate::drivers::process::ProcessState::Running => "RUNNING",
            crate::drivers::process::ProcessState::Blocked => "BLOCKED",
            crate::drivers::process::ProcessState::Terminated => "TERMINATED",
            crate::drivers::process::ProcessState::Sleeping => "SLEEPING",
        };
        shell.write_str(&alloc::format!("  {:>4}  {:16}  {:10}  ---  (preemptive/fringe)\n", id, name, state_str));
    }
    shell.write_str(&alloc::format!("\nTotal: {} process(es), preemptive scheduler (fringe)\n", procs.len()));
}
