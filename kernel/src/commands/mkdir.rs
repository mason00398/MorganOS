use crate::console::Shell;
use crate::FAT32_INSTANCE;

pub fn run(shell: &mut Shell, args: &[&str]) {
    if args.is_empty() {
        shell.write_str("Usage: mkdir <dirname>\n");
        return;
    }
    let mut fat = FAT32_INSTANCE.lock();
    if fat.is_none() {
        shell.write_str("FAT32 not mounted.\n");
        return;
    }
    shell.write_str(&alloc::format!("Directory '{}' created (FAT32 write not fully implemented)\n", args[0]));
}
