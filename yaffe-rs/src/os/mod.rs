use crate::input::PlatformGamepad;
use std::convert::From;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod os_impl;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "linux.rs"]
mod os_impl;

#[derive(Debug)]
#[allow(dead_code)]
pub enum PlatformError {
    AccessDenied,
    File(std::io::Error),
    Other(String),
}

impl From<std::io::Error> for PlatformError {
    fn from(v: std::io::Error) -> Self { PlatformError::File(v) }
}

impl From<String> for PlatformError {
    fn from(v: String) -> Self { PlatformError::Other(v) }
}

type PlatformResult<T> = Result<T, PlatformError>;

pub fn lib_ext() -> &'static str { os_impl::lib_ext() }

pub fn app_ext() -> &'static str { os_impl::app_ext() }

pub fn shutdown() -> PlatformResult<()> { os_impl::shutdown() }

pub fn set_run_at_startup(task: &str, value: bool) -> PlatformResult<()> {
    Ok(os_impl::set_run_at_startup(task, value).unwrap())
}

pub fn get_run_at_startup(task: &str) -> PlatformResult<bool> { os_impl::get_run_at_startup(task) }

pub fn get_volume() -> PlatformResult<f32> { os_impl::get_volume() }

pub fn set_volume(delta: f32) -> PlatformResult<()> { os_impl::set_volume(delta) }

pub fn initialize_gamepad() -> Result<impl PlatformGamepad, i32> { Ok(os_impl::initialize_gamepad().unwrap()) }

pub fn sanitize_file(file: &str) -> String { os_impl::sanitize_file(file) }
