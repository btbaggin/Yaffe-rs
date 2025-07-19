use std::collections::HashMap;

use crate::PluginError;

#[repr(C)]
#[derive(Clone, PartialEq)]
pub enum SettingValue {
    String(String),
    F32(f32),
    I32(i32),
    Tuple((f32, f32, f32, f32)), // RGBA
}
impl std::fmt::Display for SettingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingValue::Tuple(c) => write!(f, "({:.1},{:.1},{:.1},{:.1})", c.0, c.1, c.2, c.3),
            SettingValue::F32(ff) => write!(f, "{ff:.1}"),
            SettingValue::I32(i) => write!(f, "{i}"),
            SettingValue::String(s) => write!(f, "{s}"),
        }
    }
}

pub type SettingsResult<T> = Result<T, SettingLoadError>;

#[repr(C)]
#[derive(Debug)]
pub enum SettingLoadError {
    IncorrectFormat,
    InvalidType(String),
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

pub trait PluginSettings {
    fn try_get_str(&self, name: &str) -> Option<String>;
    fn get_str(&self, name: &str) -> Result<String, PluginError>;
    fn try_get_i32(&self, name: &str) -> Option<i32>;
    fn get_i32(&self, name: &str) -> Result<i32, PluginError>;
    fn try_get_f32(&self, name: &str) -> Option<f32>;
    fn get_f32(&self, name: &str) -> Result<f32, PluginError>;
}
impl PluginSettings for HashMap<String, SettingValue> {
    fn try_get_str(&self, name: &str) -> Option<String> {
        if let Some(SettingValue::String(s)) = self.get(name) {
            return Some(s.clone());
        }
        None
    }
    fn get_str(&self, name: &str) -> Result<String, PluginError> {
        self.try_get_str(name).ok_or(PluginError::MissingSetting(name.to_string()))
    }

    fn try_get_i32(&self, name: &str) -> Option<i32> {
        if let Some(SettingValue::I32(s)) = self.get(name) {
            return Some(*s);
        }
        None
    }
    fn get_i32(&self, name: &str) -> Result<i32, PluginError> {
        self.try_get_i32(name).ok_or(PluginError::MissingSetting(name.to_string()))
    }

    fn try_get_f32(&self, name: &str) -> Option<f32> {
        if let Some(SettingValue::F32(s)) = self.get(name) {
            return Some(*s);
        }
        None
    }
    fn get_f32(&self, name: &str) -> Result<f32, PluginError> {
        self.try_get_f32(name).ok_or(PluginError::MissingSetting(name.to_string()))
    }
}
