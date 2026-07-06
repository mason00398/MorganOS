use crate::console::Shell;

/// whoami - 显示当前用户
pub fn run(shell: &mut Shell) {
    shell.write_str("root\n");
}
