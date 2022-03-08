use std::fs::{OpenOptions, File};
use std::io::Write;
use std::sync::Mutex;
use std::fmt::Debug;
use std::convert::TryFrom;

#[repr(u8)]
#[allow(dead_code)]
#[derive(PartialOrd, PartialEq, Clone, Copy)]
pub enum LogTypes {
    Fine,
    Information,
    Warning,
    Error,
}
impl TryFrom<i32> for LogTypes {
    type Error = ();
    fn try_from(level: i32) -> Result<Self, ()> {
        match level {
            0 => Ok(LogTypes::Fine),
            1 => Ok(LogTypes::Information),
            2 => Ok(LogTypes::Warning),
            _ => Ok(LogTypes::Error),
        }
    }
}

pub struct Logger {
    file: Mutex<File>,
    level: Mutex<LogTypes>,
}
impl Logger {
    fn new(path: &'static str) -> Logger {
        Logger {
            file: Mutex::new(OpenOptions::new().create(true).write(true).open(path).unwrap()),
            level: Mutex::new(LogTypes::Fine),
        }
    }
    fn set_level(&self, level: LogTypes) {
        let mut self_level = self.level.lock().unwrap();
        *self_level = level;
    }
    fn level(&self) -> LogTypes {
        *self.level.lock().unwrap()
    }
}

pub fn set_log_level(level: i32) {
    use std::convert::TryInto;
    let level = level.try_into().unwrap();
    LOGGER.set_level(level)
}

lazy_static::lazy_static! {
    static ref LOGGER: Logger = Logger::new("./log.txt");
}

macro_rules! log_entry_internal {
    ($type:ident, $string:expr, $($element:tt)*) => {
        let file = &LOGGER.file;
        if $type >= LOGGER.level() {
            let mut file = file.lock().unwrap();
        
            let time = chrono::Local::now();
            let time_string = time.format("%x %X");
            let message = match $type {
                //Include stack trace in debug builds
                #[cfg(debug_assertions)]
                LogTypes::Error => {
                    let trace = backtrace::Backtrace::new();
                    format!("Error [{}]: {} {:?}\n", time_string, format_args!($string, $($element)*), trace)
                },
                #[cfg(not(debug_assertions))]
                LogTypes::Error => format!("Error [{}]: {}\n", time_string, format_args!($string, $($element)*)),
                LogTypes::Warning => format!("Warning [{}]: {}\n", time_string, format_args!($string, $($element)*)),
                LogTypes::Information | LogTypes::Fine => format!("[{}]: {}\n", time_string, format_args!($string, $($element)*)),
            };
            file.write_all(message.as_bytes()).unwrap();
        } 
    }
} 

/// Logs a piece of data
pub fn log_entry(t: LogTypes, err: impl Debug) {
    log_entry_internal!(t, "{:?}", err);
}

/// Logs a piece of data long with an addtional message
pub fn log_entry_with_message(t: LogTypes, err: impl Debug, message: &str) {
    log_entry_internal!(t, "{:?} - {}", err, message);
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
                log_entry_with_message(LogTypes::Error, e, message);
                panic!("encountered unexpected error");
            }
            Ok(r) => r,
        }
    }

    /// Logs the type if it is `Err` then panics
    fn log_and_panic(self) -> T {
        match self {
            Err(e) => {
                log_entry(LogTypes::Error, e);
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
                log_entry_with_message(LogTypes::Warning, e, message);
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
                let message = format!("{}: {:?}", message, e);
                let message = Box::new(crate::modals::MessageModalContent::new(&message));
                crate::modals::display_modal(state, "Error", None, message, crate::modals::ModalSize::Half, None);
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
                let message = format!("{}: {:?}", message, e);
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
                log_entry_with_message(LogTypes::Error, "None", message);
                panic!("encountered unexpected error");
            }
        }
    }

    /// Logs the type if it is `None` then panics
    fn log_and_panic(self) -> T {
        match self {
            Some(t) => t,
            None => {
                log_entry(LogTypes::Error, "None");
                panic!("encountered unexpected error");
            }
        }
    }
}