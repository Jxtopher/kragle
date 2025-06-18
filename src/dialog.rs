use std::io::Write;

use console::Style;
use terminal_size::{Width, terminal_size};

pub enum Status {
    Unknown,
    Ok,
    Failed,
}

impl Status {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Unknown => "[?]",
            Self::Ok => "[OK]",
            Self::Failed => "[FAILED]",
        }
    }
    pub fn colorized(&self) -> String {
        match self {
            Self::Unknown => "[?]".to_string(),
            Self::Ok => Style::new().green().apply_to("[OK]").to_string(),
            Self::Failed => Style::new().red().apply_to("[FAILED]").to_string(),
        }
    }
}

pub struct Dialog {
    msg: String,
    frames: usize,
}

impl Dialog {
    fn first_n_chars(s: &str, n: usize) -> &str {
        s.char_indices()
            .nth(n)
            .map(|(idx, _)| &s[..idx])
            .unwrap_or(s)
    }

    pub fn new(msg: String) -> Self {
        Dialog { msg, frames: 0 }
    }

    pub fn get_width(&self, suffix: &str) -> usize {
        let term_width = terminal_size()
            .map(|(Width(w), _)| w as usize)
            .unwrap_or(80)
            .min(80);

        term_width.saturating_sub(suffix.len())
    }

    pub fn update_msg(&mut self, msg: String) {
        self.msg = msg;
    }

    pub fn start_print(&self, status: Status) {
        print!(
            "{:<width$}{}",
            Dialog::first_n_chars(&self.msg, 70),
            status.colorized(),
            width = self.get_width(status.to_string())
        );
        std::io::stdout().flush().unwrap();
    }

    pub fn end_print(&self, status: Status) {
        println!(
            "\r{:<width$}{}",
            Dialog::first_n_chars(&self.msg, 70),
            status.colorized(),
            width = self.get_width(status.to_string())
        );
    }

    pub fn spinner(&mut self) {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let status_str = spinner[self.frames];

        print!(
            "\r{:<width$}[{}]",
            Dialog::first_n_chars(&self.msg, 70),
            status_str,
            width = self.get_width(status_str)
        );
        std::io::stdout().flush().unwrap();
        self.frames = (self.frames + 1) % spinner.len();
    }
}
