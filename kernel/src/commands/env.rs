use crate::console::Shell;

/// env - 显示环境变量
pub fn run(shell: &mut Shell) {
    shell.write_str("KERNEL_VERSION=0.3.0\n");
    shell.write_str("BUILD_DATE=\n");
    shell.write_str("HOSTNAME=runst-x\n");
    shell.write_str("USER=root\n");
    shell.write_str("PATH=/bin:/sbin\n");
    shell.write_str("(Environment variables simulated)\n");
}
