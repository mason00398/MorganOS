use crate::console::Shell;

pub fn run(shell: &mut Shell) {
    shell.write_str(&alloc::format!("RunST X Kernel v{}\n", env!("CARGO_PKG_VERSION")));
    shell.write_str(&alloc::format!("Build: {}\n", env!("BUILD_DATE")));
    shell.write_str(&alloc::format!("Git: {}\n", env!("GIT_HASH")));
    shell.write_str("Features: FAT32 TCP/IP MultiProcess EnhancedKeyboard\n");
}
