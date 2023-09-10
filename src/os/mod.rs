use crate::input::{ControllerInput, PlatformGamepad};
use std::convert::From;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod os_impl;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "linux.rs"]
mod os_impl;

#[derive(Debug)]
#[allow(dead_code)]
pub enum StartupError {
    AccessDenied,
    File(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for StartupError {
    fn from(v: std::io::Error) -> Self {
        StartupError::File(v)
    }
}

impl From<String> for StartupError {
   fn from(v: String) -> Self {
        StartupError::Other(v)
    }
}

type StartupResult<T> = Result<T, StartupError>;
type ShutdownResult = std::io::Result<()>;
type VolumeResult<T> = Result<T, StartupError>;

pub fn update() -> std::io::Result<std::process::Child> {
    os_impl::update()
}

pub fn shutdown() -> ShutdownResult {
    os_impl::shutdown()
}

pub fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
    os_impl::set_run_at_startup(task, value)
}

pub fn get_run_at_startup(task: &str) -> StartupResult<bool> {
    os_impl::get_run_at_startup(task)
}

pub fn get_and_update_volume(delta: f32) -> VolumeResult<f32> {
    os_impl::get_and_update_volume(delta)
}

pub fn initialize_gamepad() -> Result<impl PlatformGamepad, i32> {
    os_impl::initialize_gamepad()
}

pub fn get_clipboard(window: &glutin::window::Window) -> Option<String> {
    os_impl::get_clipboard(window)
}

pub fn sanitize_file(file: &str) -> String {
    os_impl::sanitize_file(file)
}