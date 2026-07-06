use crate::console::Shell;
use crate::FAT32_INSTANCE;

pub fn run(shell: &mut Shell, args: &[&str]) {
    if args.is_empty() {
        shell.write_str("Usage: cat <filename>\n");
        return;
    }
    let mut fat = FAT32_INSTANCE.lock();
    let fat = match fat.as_mut() {
        Some(f) => f,
        None => {
            shell.write_str("FAT32 not mounted.\n");
            return;
        }
    };
    // 简化：硬编码根目录簇2查找
    let mut found = false;
    fat.read_directory(2, |name, cluster, size| {
        if name == args[0] && !found {
            found = true;
            let mut buf = alloc::vec![0u8; size as usize];
            if fat.read_file(cluster, size, &mut buf) {
                shell.write_str(core::str::from_utf8(&buf).unwrap_or("<binary>"));
                shell.write_str("\n");
            } else {
                shell.write_str("Read failed.\n");
            }
        }
    });
    if !found {
        shell.write_str(&alloc::format!("File not found: {}\n", args[0]));
    }
}
