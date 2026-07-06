use crate::console::Shell;
use crate::FAT32_INSTANCE;

pub fn run(shell: &mut Shell, args: &[&str]) {
    let path = args.first().unwrap_or(&"/");
    let mut fat = FAT32_INSTANCE.lock();
    let fat = match fat.as_mut() {
        Some(f) => f,
        None => {
            shell.write_str("FAT32 not mounted.\n");
            return;
        }
    };
    shell.write_str(&alloc::format!("Directory: {}\n", path));
    fat.read_directory(2, |name, cluster, size| {
        shell.write_str(&alloc::format!("  {} (cluster={}, size={})\n", name, cluster, size));
    });
}
