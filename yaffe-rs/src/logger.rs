use crate::DeferredAction;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::sync::{Mutex, LazyLock};

#[allow(unused_imports)]
pub use log::{debug, error, info, trace, warn};
use log::{Level, LevelFilter, Log, Metadata, Record};

struct YaffeLogger;
impl Log for YaffeLogger {
    fn enabled(&self, _: &Metadata) -> bool { true }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            use std::io::Write;

            let time_string = chrono::Local::now().format("%x %X");
            let level = record.level();
            let mut message = format!("{}: {} [{}]: {}", record.metadata().target(), level, time_string, record.args());
            if let Level::Error = level {
                message.push_str(&format!("{:?}", std::backtrace::Backtrace::force_capture()));
            };
            message.push('\n');

            let mut file = FILE.lock().unwrap();
            file.write_all(message.as_bytes()).unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn set_log_level(level: &str) {
    use core::str::FromStr;
    let level = LevelFilter::from_str(level).log_and_panic();
    log::set_max_level(level)
}

static LOGGER: YaffeLogger = YaffeLogger;
pub static FILE: LazyLock<Mutex<File>> = LazyLock::new(|| {
    Mutex::new(OpenOptions::new().write(true).create(true).truncate(true).open("./log.txt").unwrap())
});

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

pub trait PanicLogEntry<T> {
    fn log_message_and_panic(self, message: &str) -> T;
    fn log_and_panic(self) -> T;
}
pub trait LogEntry<T: Default> {
    fn log(self, message: &str) -> T;
}
pub trait UserMessage<T> {
    fn display_failure<S: 'static>(self, message: &str, handle: &mut DeferredAction<S>) -> Option<T>;
}
impl<T, E: Debug> PanicLogEntry<T> for Result<T, E> {
    /// Logs the type with an additional message if it is `Err` then panics  
    fn log_message_and_panic(self, message: &str) -> T {
        match self {
            Err(e) => {
                log::error!("{e:?} - {message}");
                panic!("encountered unexpected error");
            }
            Ok(r) => r,
        }
    }

    /// Logs the type if it is `Err` then panics
    fn log_and_panic(self) -> T {
        match self {
            Err(e) => {
                log::error!("{e:?}");
                panic!("encountered unexpected error");
            }
            Ok(r) => r,
        }
    }
}

impl<T: Default, E: Debug> LogEntry<T> for Result<T, E> {
    fn log(self, message: &str) -> T {
        match self {
            Err(e) => {
                log::warn!("{e:?} - {message}");
                Default::default()
            }
            Ok(r) => r,
        }
    }
}

impl<T, E: Debug> UserMessage<T> for Result<T, E> {
    /// Displays a message to the user, but can be called with a DeferredAction when access there is no access to YaffeState
    /// Returns `Some(T)` when there was no error, otherwise `None`
    fn display_failure<S: 'static>(self, message: &str, handle: &mut DeferredAction<S>) -> Option<T> {
        match self {
            Err(e) => {
                let message = format!("{message}: {e:?}");
                handle.display_message(message);
                None
            }
            Ok(r) => Some(r),
        }
    }
}

impl<T> PanicLogEntry<T> for Option<T> {
    /// Logs the type with an additional message if it is `None` then panics
    fn log_message_and_panic(self, message: &str) -> T {
        match self {
            Some(t) => t,
            None => {
                log::error!("None - {message}");
                panic!("encountered unexpected error");
            }
        }
    }

    /// Logs the type if it is `None` then panics
    fn log_and_panic(self) -> T {
        match self {
            Some(t) => t,
            None => {
                log::error!("None",);
                panic!("encountered unexpected error");
            }
        }
    }
}
