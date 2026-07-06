use alloc::string::String;
use crate::console::Shell;

mod ver;
mod help;
mod reboot;
mod shutdown;
mod echo;
mod clear;
mod meminfo;
mod uptime;
mod test;
mod pcilist;
mod net;
mod ls;
mod cat;
mod mkdir;
mod ps;
mod kill_cmd;
mod ifconfig;
mod ping;
mod date_cmd;
mod cal;
mod whoami;
mod env;
mod vmstat;

pub fn dispatch(shell: &mut Shell, line: String) {
    if line.is_empty() {
        return;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    let cmd = parts[0];
    let args: Vec<&str> = parts[1..].to_vec();

    match cmd {
        "ver" => ver::run(shell),
        "help" => help::run(shell, args.first().copied()),
        "reboot" => reboot::run(shell),
        "shutdown" => shutdown::run(shell),
        "echo" => echo::run(shell, &args),
        "clear" => clear::run(shell),
        "meminfo" => meminfo::run(shell),
        "uptime" | "time" => uptime::run(shell),
        "test" => test::run(shell),
        "pcilist" => pcilist::run(shell),
        "net" => net::run(shell),
        "ls" | "dir" => ls::run(shell, &args),
        "cat" => cat::run(shell, &args),
        "mkdir" => mkdir::run(shell, &args),
        "ps" => ps::run(shell),
        "kill" => kill_cmd::run(shell, &args),
        "ifconfig" | "ip" => ifconfig::run(shell),
        "ping" => ping::run(shell, &args),
        "date" => date_cmd::run(shell),
        "cal" => cal::run(shell),
        "whoami" => whoami::run(shell),
        "env" => env::run(shell),
        "vmstat" => vmstat::run(shell),
        "cls" => clear::run(shell),
        _ => shell.write_str(&alloc::format!("Unknown command: {}\nType 'help' for commands.\n", cmd)),
    }
}
