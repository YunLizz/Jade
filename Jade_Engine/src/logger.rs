#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn label(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

pub struct Logger {
    entries: Vec<LogEntry>,
    show_timestamp: bool,
    show_level: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            show_timestamp: true,
            show_level: true,
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let now = chrono_like_now();
        self.entries.push(LogEntry {
            timestamp: now,
            level,
            message: message.into(),
        });
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Error, message);
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn show_timestamp(&self) -> bool {
        self.show_timestamp
    }

    pub fn set_show_timestamp(&mut self, value: bool) {
        self.show_timestamp = value;
    }

    pub fn show_level(&self) -> bool {
        self.show_level
    }

    pub fn set_show_level(&mut self, value: bool) {
        self.show_level = value;
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

fn chrono_like_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let total = secs % 86400;
    let hours = total / 3600;
    let mins = (total % 3600) / 60;
    let secs = total % 60;

    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_millis())
        .unwrap_or(0);

    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms)
}
