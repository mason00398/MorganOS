use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::drivers::vga::VgaWriter;
use crate::drivers::keyboard::{key_pressed, read_char};

pub struct Shell {
    input: Vec<char>,
    history: Vec<String>,
    history_index: usize,
    vga: VgaWriter,
}

impl Shell {
    pub fn new() -> Self {
        let mut vga = VgaWriter::new();
        vga.set_color(crate::drivers::vga::Color::LightGreen, crate::drivers::vga::Color::Black);
        Shell {
            input: Vec::new(),
            history: Vec::new(),
            history_index: 0,
            vga,
        }
    }

    pub fn print_prompt(&self) {
        self.vga.write_str("RunST X> ");
    }

    pub fn read_line(&mut self) -> Option<String> {
        self.input.clear();
        loop {
            if key_pressed() {
                if let Some(ch) = read_char() {
                    match ch {
                        '\n' => {
                            self.vga.write_str("\n");
                            let line: String = self.input.iter().collect();
                            if !line.is_empty() {
                                self.history.push(line.clone());
                                self.history_index = self.history.len();
                            }
                            return Some(line);
                        }
                        '\x08' | '\x7F' => {
                            if !self.input.is_empty() {
                                self.input.pop();
                                self.vga.write_str("\u{8} \u{8}");
                            }
                        }
                        // ===== 修复 #43：Tab补全框架 =====
                        '\t' => {
                            let current: String = self.input.iter().collect();
                            let matches: Vec<&str> = ["help", "ver", "reboot", "shutdown", "echo", "clear", "meminfo", "uptime", "test", "pcilist", "net", "ls", "cat", "mkdir", "ps", "kill", "ifconfig", "ping", "date", "cal", "whoami", "env"]
                                .iter()
                                .filter(|cmd| cmd.starts_with(&current))
                                .map(|s| *s)
                                .collect();
                            if matches.len() == 1 {
                                self.input.clear();
                                self.input.extend(matches[0].chars());
                                self.vga.write_str(&format!("\rRunST X> {}", matches[0]));
                            } else if matches.len() > 1 {
                                self.vga.write_str("\n");
                                for m in matches {
                                    self.vga.write_str(&format!("  {}\n", m));
                                }
                                self.print_prompt();
                                let current_str: String = self.input.iter().collect();
                                self.vga.write_str(&current_str);
                            }
                        }
                        // ===== 修复 #42：上下键历史 =====
                        '\x1B' => {
                            // ANSI转义序列：方向键
                            if let Some('[') = read_char() {
                                match read_char() {
                                    Some('A') => { // 上箭头
                            // 方向键产生的是 ESC [ A/B/C/D 序列
                            // 可以在这里触发命令历史上下翻
                                        if self.history_index > 0 {
                                            self.history_index -= 1;
                                            self.input.clear();
                                            self.input.extend(self.history[self.history_index].chars());
                                            self.vga.write_str(&format!("\rRunST X> {}", self.history[self.history_index]));
                                        }
                                    }
                                    Some('B') => { // 下箭头
                                        if self.history_index < self.history.len() - 1 {
                                            self.history_index += 1;
                                            self.input.clear();
                                            self.input.extend(self.history[self.history_index].chars());
                                            self.vga.write_str(&format!("\rRunST X> {}", self.history[self.history_index]));
                                        } else if self.history_index == self.history.len() - 1 {
                                            self.history_index = self.history.len();
                                            self.input.clear();
                                            self.vga.write_str(&format!("\rRunST X> "));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        c => {
                            self.input.push(c);
                            self.vga.write_str(&c.to_string());
                        }
                    }
                }
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.vga.write_str(s);
    }

    pub fn write_fmt(&mut self, args: alloc::fmt::Arguments) {
        struct Writer<'a>(&'a mut VgaWriter);
        impl alloc::fmt::Write for Writer<'_> {
            fn write_str(&mut self, s: &str) -> alloc::fmt::Result {
                self.0.write_str(s);
                Ok(())
            }
        }
        let _ = Writer(&mut self.vga).write_fmt(args);
    }

    pub fn clear_screen(&mut self) {
        self.vga.clear();
    }
}
