use crate::console::Shell;

pub fn run(shell: &mut Shell) {
    shell.write_str("Shutting down...\n");
    shell.write_str("It is safe to power off now.\n");
    loop {}
}
