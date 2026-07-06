use crate::console::Shell;
use crate::drivers::rtc::Rtc;

/// date - 显示/设置日期
pub fn run(shell: &mut Shell) {
    let time = Rtc::get_datetime_string();
    shell.write_str(&alloc::format!("Current date/time: {}\n", time));
    shell.write_str("(Set not implemented)\n");
}
