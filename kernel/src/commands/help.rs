use crate::console::Shell;

pub fn run(shell: &mut Shell, arg: Option<&str>) {
    let cmds = [
        ("ver", "Show version info"),
        ("help", "Show this help"),
        ("reboot", "Reboot system"),
        ("shutdown", "Shutdown system"),
        ("echo", "Echo text: echo hello"),
        ("clear", "Clear screen"),
        ("meminfo", "Show memory & VM usage"),
        ("vmstat", "Show VM & process stats"),
        ("uptime", "Show current time"),
        ("test", "Run hardware diagnostics"),
        ("pcilist", "List PCI devices"),
        ("net", "Network: net status/ping/socket/dns"),
        ("ls", "List directory: ls [path]"),
        ("cat", "Show file content: cat filename"),
        ("mkdir", "Create directory: mkdir dirname"),
        ("ps", "List running processes"),
        ("kill", "Kill process: kill PID"),
        ("ifconfig", "Show network config"),
        ("ping", "Ping host: ping 192.168.1.1 [count]"),
        ("date", "Show/set date"),
        ("cal", "Show calendar"),
        ("whoami", "Show current user"),
        ("env", "Show environment variables"),
    ];

    if let Some(cmd) = arg {
        for (name, desc) in &cmds {
            if *name == cmd {
                shell.write_str(&alloc::format!("{}: {}\n", name, desc));
                return;
            }
        }
        shell.write_str(&alloc::format!("No help for '{}'\n", cmd));
        return;
    }

    shell.write_str("Available commands:\n");
    for (name, desc) in &cmds {
        shell.write_str(&alloc::format!("  {:12} {}\n", name, desc));
    }
}
