use std::fs::{OpenOptions, File};
use std::sync::Mutex;
use std::fmt::Debug;

pub use log::{error, warn, info, trace};
use log::{Log, Level, LevelFilter, Metadata, Record};

struct YaffeLogger;
impl Log for YaffeLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

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

    fn flush(&self) { }
}

pub fn set_log_level(level: &str) {
    use core::str::FromStr;
    let level = LevelFilter::from_str(level).log_and_panic();
    log::set_max_level(level)
}

static LOGGER: YaffeLogger = YaffeLogger;
lazy_static::lazy_static! {
    pub static ref FILE: Mutex<File> = Mutex::new(OpenOptions::new().create(true).write(true).open("./log.txt").unwrap());
}

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
    fn display_failure(self, message: &str, state: &mut crate::YaffeState) -> Option<T>;
    fn display_failure_deferred(self, message: &str, handle: &mut crate::DeferredAction) -> Option<T>;
}
impl<T, E: Debug> PanicLogEntry<T> for std::result::Result<T, E> {
    /// Logs the type with an additional message if it is `Err` then panics  
    fn log_message_and_panic(self, message: &str) -> T {
        match self {
            Err(e) => {
                log::error!("{:?} - {}", e, message);
                panic!("encountered unexpected error");
            }
            Ok(r) => r,
        }
    }

    /// Logs the type if it is `Err` then panics
    fn log_and_panic(self) -> T {
        match self {
            Err(e) => {
                log::error!("{:?}", e);
                panic!("encountered unexpected error");
            }
            Ok(r) => r,
        }
    }
}

impl <T: Default, E: Debug> LogEntry<T> for std::result::Result<T, E> {
    fn log(self, message: &str) -> T {
        match self {
            Err(e) => {
                log::warn!("{:?} - {}", e, message);
                std::default::Default::default()
            }
            Ok(r) => r,
        }
    }
}

impl<T, E: Debug> UserMessage<T> for std::result::Result<T, E> {
    /// Displays a message to the user if it is `Err`
    /// Returns `Some(T)` when there was no error, otherwise `None`
    fn display_failure(self, message: &str, state: &mut crate::YaffeState) -> Option<T> {
        match self {
            Err(e) => {
                let message = format!("{message}: {e:?}");
                let message = Box::new(crate::modals::MessageModalContent::new(&message));
                crate::ui::display_modal(state, "Error", None, message, None);
                None
            }
            Ok(r) => Some(r),
        }
    }

    /// Displays a message to the user, but can be called with a DeferredAction when access there is no access to YaffeState
    /// Returns `Some(T)` when there was no error, otherwise `None`
    fn display_failure_deferred(self, message: &str, handle: &mut crate::DeferredAction) -> Option<T> {
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
                log::error!("None - {}", message);
                panic!("encountered unexpected error");
            }
        }
    }

    /// Logs the type if it is `None` then panics
    fn log_and_panic(self) -> T {
        match self {
            Some(t) => t,
            None => {
                log::error!("None", );
                panic!("encountered unexpected error");
            }
        }
    }
}