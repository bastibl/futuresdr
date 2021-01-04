use log::{LevelFilter, Metadata, Record};

use crate::runtime::config;

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("FutureSDR: {} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_boxed_logger(Box::new(Logger)).expect("failed to set logger");
    let log_level : String = config::get_or_default("log_level", "debug".to_string());

    match &log_level[..] {
        "off" => log::set_max_level(LevelFilter::Off),
        "error" => log::set_max_level(LevelFilter::Error),
        "warn" => log::set_max_level(LevelFilter::Warn),
        "info" => log::set_max_level(LevelFilter::Info),
        "trace" => log::set_max_level(LevelFilter::Trace),
        _ => log::set_max_level(LevelFilter::Debug),
    }
}
