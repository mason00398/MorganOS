use crate::console::Shell;

pub fn run(shell: &mut Shell, args: &[&str]) {
    shell.write_str(&alloc::format!("{}\n", args.join(" ")));
}
