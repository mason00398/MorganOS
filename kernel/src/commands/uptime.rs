use crate::console::Shell;
use crate::drivers::rtc::Rtc;

pub fn run(shell: &mut Shell) {
    let time = Rtc::get_datetime_string();
    shell.write_str(&alloc::format!("{}\n", time));
}
