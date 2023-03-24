use log::{info, Level, LevelFilter, Log, Metadata, Record};

use crate::{arch::get_cpu_id, println};

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color_code = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 34,  // Blue
            Level::Debug => 32, // Green
            Level::Trace => 33, // BrightBlack
        };
        let cpu_id = get_cpu_id();
        println!(
            "\u{1B}[{}m[CPU{:>3}][{:>5}] ({}:{}) {} \u{1B}[0m",
            color_code,
            cpu_id,
            record.level(),
            record.file().unwrap(),
            record.line().unwrap(),
            record.args(),
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).expect("Failed to initialize logger");
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });
    info!("Console logger successfully initialized.")
}
