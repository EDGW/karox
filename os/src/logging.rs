use crate::{
    backgr, bright,
    console::{
        colors::{BLACK, GREEN, RED, WHITE, YELLOW},
        styles::ITALIC,
    },
    panic_init,
};
use log::{Level, LevelFilter, Log, Metadata, Record, set_logger, set_max_level};

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color_str = match record.level() {
            Level::Error => format_args!("{}{}", ansi_color!(WHITE), ansi_color!(backgr!(RED))), // Red
            Level::Warn => ansi_color!(ITALIC, YELLOW),
            Level::Info => ansi_color!(WHITE),           // White
            Level::Debug => ansi_color!(GREEN),          // Green
            Level::Trace => ansi_color!(bright!(BLACK)), // BrightBlack
        };
        kserial_println!(
            "{}[{:}] {}\u{1B}[0m",
            color_str,
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
    (logger: $logger:expr, target: $target:expr, $fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(logger: __log_logger!($logger), target: $target, Level::Debug, concat!("[Hart #{}]",$fmt),$crate::arch::hart::get_current_hart_id() $(, $($arg)+)?)
        }
    };

    // debug!(logger: my_logger, key1 = 42, key2 = true; "a {} event", "log")
    // debug!(logger: my_logger, "a {} event", "log")
    (logger: $logger:expr, $fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(logger: __log_logger!($logger), Level::Debug, concat!("[Hart #{}]",$fmt),$crate::arch::hart::get_current_hart_id() $(, $($arg)+)?)
        }
    };

    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(target: $target, Level::Debug, concat!("[Hart #{}]",$fmt),$crate::arch::hart::get_current_hart_id() $(, $($arg)+)?)
        }
    };

    // debug!("a {} event", "log")
    ($fmt: literal $(, $($arg: tt)+)?) => {
        #[cfg(debug_assertions)]
        {
            use log::{log,Level};
            log!(Level::Debug, concat!("(#{}) ", $fmt),$crate::arch::hart::get_current_hart_id() $(, $($arg)+)?)
        }
    };
}
