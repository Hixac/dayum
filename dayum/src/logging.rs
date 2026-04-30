use log::{Level, LevelFilter, Log, Metadata, Record, set_logger, set_max_level};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Enable all levels – you could add filtering here
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    set_logger(&SimpleLogger).unwrap();
    set_max_level(LevelFilter::Info);
}
