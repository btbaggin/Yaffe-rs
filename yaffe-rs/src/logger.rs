use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::fmt::Debug;

#[repr(u8)]
#[allow(dead_code)]
pub enum LogTypes {
    Error,
    Warning,
    Information,
}

static mut LOG_FILE: Option<Mutex<File>> = None;

pub fn initialize_log() {
    unsafe {
        LOG_FILE = Some(Mutex::new(OpenOptions::new().create(true).append(true).open("./log.txt").unwrap()));
    }
}

macro_rules! log_entry_internal {
    ($type:ident, $string:expr, $($element:tt)*) => {
        let file = unsafe { &LOG_FILE.as_ref().unwrap() };
        let mut file = file.lock().unwrap();
    
        let time = chrono::Local::now();
        let time_string = time.format("%x %X");
        let message = match $type {
            #[cfg(debug_assertions)]
            LogTypes::Error => {
                let trace = backtrace::Backtrace::new();
                format!("Error {{{}}}: {} {:?}\n", time_string, format_args!($string, $($element)*), trace)
            },
            #[cfg(not(debug_assertions))]
            LogTypes::Error => format!("Error {{{}}}: {}\n", time_string, format_args!($string, $($element)*)),
            LogTypes::Warning => format!("Warning {{{}}}: {}\n", time_string, format_args!($string, $($element)*)),
            LogTypes::Information => format!("Info {{{}}}: {}\n", time_string, format_args!($string, $($element)*)),
        };
        file.write_all(message.as_bytes()).unwrap();
    }
} 

#[macro_export]
macro_rules! log_function {
    ($($parm:ident),*) => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }

        // Find and cut the rest of the path
        let name = type_name_of(f);
        let mut name = String::from(&name[..name.len() - 3]);
        
        name.push_str("(");
        $(
            name.push_str(&format!("{:?},", $parm));
        )*
        let mut name = String::from(name.trim_matches(','));
        name.push_str(")");

        crate::logger::log_entry(crate::logger::LogTypes::Information, &name);
    }};
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
    fn log_if_fail(self, message: &str) -> T;
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
    fn log_if_fail(self, message: &str) -> T {
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