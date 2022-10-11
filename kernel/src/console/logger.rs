use core::fmt;
use log::{info, Log, Metadata, Record};
use once_cell::sync::Lazy;

use crate::println;

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
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
            Level::Trace => 90, // BrightBlack
        };
        println!(
            "\u{1B}[{color_code}m[{:>5}] {}\u{1B}[0m",
            record.level(),
            record.args(),
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER);
    log::set_max_level(option_env!("LOG"));
    info!("Initialize console logger successfully.")
}
