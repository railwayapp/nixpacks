use colored::Colorize;

/// Used for reporting Docker build information to stdout.
pub struct Logger {}

impl Logger {
    pub fn new() -> Logger {
        Logger {}
    }

    /// Pretty-print the given log section title.
    pub fn log_section(&self, msg: &str) {
        println!("=== {} ===", msg.magenta().bold());
    }

    /// Pretty-print the given log line.
    pub fn log_step(&self, msg: &str) {
        println!("=> {msg}");
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
