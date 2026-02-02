use log::{Level, LevelFilter, Log, Metadata, Record, set_logger, set_max_level};

use crate::panic_init;

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 20,  // White
            Level::Debug => 32, // Green
            Level::Trace => 90, // BrightBlack
        };
        kserial_println!(
            "\u{1B}[{}m[{:}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }

    fn flush(&self) {}
}

pub fn init() {
    static LOGGER: Logger = Logger;
    set_logger(&LOGGER).unwrap_or_else(|err| panic_init!("Error initializing logger: {:?}", err));
    set_max_level(LevelFilter::Debug);
}

/// Improved debug macro,
/// only compiled in debug mode.
#[macro_export]
macro_rules! debug_ex {
    // debug!(logger: my_logger, target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(logger: my_logger, target: "my_target", "a {} event", "log")
    (logger: $logger:expr, target: $target:expr, $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(logger: __log_logger!($logger), target: $target, Level::Debug, $($arg)+)
        }
    };

    // debug!(logger: my_logger, key1 = 42, key2 = true; "a {} event", "log")
    // debug!(logger: my_logger, "a {} event", "log")
    (logger: $logger:expr, $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(logger: __log_logger!($logger), Level::Debug, $($arg)+)
        }
    };

    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(target: $target, Level::Debug, $($arg)+)
        }
    };

    // debug!("a {} event", "log")
    ($($arg:tt)+) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(Level::Debug, $($arg)+)
        }
    }
}
