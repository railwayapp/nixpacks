use colored::Colorize;

pub struct Logger {}

impl Logger {
    pub fn new() -> Logger {
        Logger {}
    }

    pub fn log_section(&self, msg: &str) {
        println!("\n=== {} ===", msg.magenta().bold());
    }

    pub fn log_step(&self, msg: &str) {
        println!("  →  {}", msg);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
