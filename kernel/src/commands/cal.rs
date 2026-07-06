use crate::console::Shell;

/// cal - 实现完整的月历（基姆拉尔森公式）
pub fn run(shell: &mut Shell, args: &[&str]) {
    let (month, year) = parse_args(args);
    print_calendar(shell, month, year);
}

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn print_calendar(shell: &mut Shell, month: u8, year: u32) {
    let days = get_days_in_month(month, year);
    let first_day = get_first_day_of_month(month, year);

    shell.write_str(&alloc::format!("    {} {}\n", MONTH_NAMES[(month - 1) as usize], year));
    shell.write_str("Su Mo Tu We Th Fr Sa\n");

    for _ in 0..first_day {
        shell.write_str("   ");
    }

    for day in 1..=days {
        shell.write_str(&alloc::format!("{:2} ", day));
        if (first_day + day) % 7 == 0 {
            shell.write_str("\n");
        }
    }
    shell.write_str("\n");
}

fn get_days_in_month(month: u8, year: u32) -> u8 {
    if month == 2 && is_leap_year(year) { 29 }
    else { DAYS_IN_MONTH[(month - 1) as usize] }
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn get_first_day_of_month(month: u8, year: u32) -> u8 {
    let m = if month < 3 { month as i32 + 12 } else { month as i32 };
    let y = if month < 3 { (year - 1) as i32 } else { year as i32 };
    let d = 1;
    let w = (d + 2 * m + 3 * (m + 1) / 5 + y + y / 4 - y / 100 + y / 400) % 7;
    w as u8
}

fn parse_args(args: &[&str]) -> (u8, u32) {
    let (mut month, mut year) = get_current_date();
    if args.len() >= 1 {
        if let Ok(m) = args[0].parse::<u8>() {
            if m >= 1 && m <= 12 { month = m; }
        }
    }
    if args.len() >= 2 {
        if let Ok(y) = args[1].parse::<u32>() {
            year = y;
        }
    }
    (month, year)
}

fn get_current_date() -> (u8, u32) {
    let (_, _, _, day, month, year) = crate::drivers::rtc::Rtc::get_time();
    (month, 2000 + year as u32)
}
