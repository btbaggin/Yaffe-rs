#[repr(C)]
#[derive(Clone)]
pub enum SettingValue {
    String(String),
    F32(f32),
    I32(i32),
    Color((f32, f32, f32, f32)), // RGBA
}

pub type SettingsResult<T> = Result<T, SettingLoadError>;

#[repr(C)]
#[derive(Debug)]
pub enum SettingLoadError {
    IncorrectFormat,
    InvalidType,
    InvalidValue,
    IoError(std::io::Error),
}
impl From<std::io::Error> for SettingLoadError {
    fn from(error: std::io::Error) -> Self { SettingLoadError::IoError(error) }
}
impl From<std::num::ParseIntError> for SettingLoadError {
    fn from(_: std::num::ParseIntError) -> Self { SettingLoadError::InvalidValue }
}
impl From<std::num::ParseFloatError> for SettingLoadError {
    fn from(_: std::num::ParseFloatError) -> Self { SettingLoadError::InvalidValue }
}
