use crate::input::ControllerInput;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod os;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "linux.rs"]
mod os;


type StartupResult<T> = Result<T, (&'static str, i32)>;
type ShutdownResult = std::io::Result<()>;

pub fn shutdown() -> ShutdownResult {
    os::shutdown()
}

pub fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
    os::set_run_at_startup(task, value)
}

pub fn get_run_at_startup(task: &str) -> StartupResult<bool> {
    os::get_run_at_startup(task)
}

pub fn initialize_gamepad() -> Result<impl crate::input::PlatformGamepad, i32> {
    os::initialize_gamepad()
}

pub fn get_clipboard() -> Option<String> {
    os::get_clipboard()
}