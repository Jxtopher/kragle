use std::io::Write;

use console::Style;
use terminal_size::{Width, terminal_size};

pub enum Status {
    Unknown,
    Ok,
    Failed,
    Warning,
}

impl Status {
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Unknown => "[?]",
            Self::Ok => "[OK]",
            Self::Failed => "[FAILED]",
            Self::Warning => "[WAR]",
        }
    }
    pub fn colorized(&self) -> String {
        match self {
            Self::Unknown => "[?]".to_string(),
            Self::Ok => Style::new().green().apply_to("[OK]").to_string(),
            Self::Failed => Style::new().red().apply_to("[FAILED]").to_string(),
            Self::Warning => Style::new().yellow().apply_to("[WAR]").to_string(),
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

    pub fn set_msg(&mut self, msg: String) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_to_string() {
        assert_eq!(Status::Unknown.to_string(), "[?]");
        assert_eq!(Status::Ok.to_string(), "[OK]");
        assert_eq!(Status::Failed.to_string(), "[FAILED]");
    }

    #[test]
    fn test_status_colorized() {
        assert_eq!(Status::Unknown.colorized(), "[?]".to_string());
        assert_eq!(
            Status::Ok.colorized(),
            Style::new().green().apply_to("[OK]").to_string()
        );
        assert_eq!(
            Status::Failed.colorized(),
            Style::new().red().apply_to("[FAILED]").to_string()
        );
    }

    #[test]
    fn test_first_n_chars() {
        assert_eq!(Dialog::first_n_chars("Hello, world!", 5), "Hello");
        assert_eq!(Dialog::first_n_chars("Rust", 10), "Rust");
    }

    #[test]
    fn test_get_width() {
        let dialog = Dialog {
            msg: String::new(),
            frames: 0,
        };
        let suffix = "[OK]";
        assert_eq!(dialog.get_width(suffix), 80 - suffix.len());
    }

    #[test]
    fn test_set_msg() {
        let mut dialog = Dialog {
            msg: String::from("Original"),
            frames: 0,
        };
        dialog.set_msg(String::from("Updated"));
        assert_eq!(dialog.msg, "Updated");
    }

    #[test]
    fn test_start_print() {
        let dialog = Dialog {
            msg: String::from("Test message"),
            frames: 0,
        };
        let status = Status::Ok;
        dialog.start_print(status);
        // Note: start_print uses stdout, so we can't easily assert the output here.
    }

    #[test]
    fn test_end_print() {
        let dialog = Dialog {
            msg: String::from("Test message"),
            frames: 0,
        };
        let status = Status::Ok;
        dialog.end_print(status);
        // Note: end_print uses stdout, so we can't easily assert the output here.
    }

    #[test]
    fn test_spinner() {
        let mut dialog = Dialog::new(String::from("Test message"));
        dialog.spinner();
        // Note: spinner updates frames, so we can't easily assert the output here.
    }
}
