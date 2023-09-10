use std::{collections::HashMap, path::PathBuf};
use speedy2d::color::Color;

#[repr(C)]
#[derive(Debug, Clone)]
pub enum PathType {
    File(PathBuf),
    Url(PathBuf),
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
    fn from(error: std::io::Error) -> Self {
        SettingLoadError::IoError(error)
    }
}
impl From<std::num::ParseIntError> for SettingLoadError {
    fn from(_: std::num::ParseIntError) -> Self {
        SettingLoadError::InvalidValue
    }
}
impl From<std::num::ParseFloatError> for SettingLoadError {
    fn from(_: std::num::ParseFloatError) -> Self {
        SettingLoadError::InvalidValue
    }
}

#[repr(C)]
#[derive(Clone)]
pub enum SettingValue {
    String(String),
    F32(f32),
    I32(i32),
    Color(Color),
}
impl SettingValue {
    pub fn from_string(&self, value: &str, allow_clear: bool) -> Result<Option<SettingValue>, SettingLoadError> {
        if value.is_empty() { return Ok(None); }

        let value = match self {
            SettingValue::Color(c) => {
                let v = color_from_string(value)?;
                if &v == c && allow_clear { None }
                else { Some(SettingValue::Color(v)) }
            },
            SettingValue::F32(f) => {
                let v = value.parse::<f32>()?; 
                if &v == f && allow_clear { None }
                else { Some(SettingValue::F32(v)) }
            },
            SettingValue::I32(i) => {
                let v = value.parse::<i32>()?; 
                if &v == i && allow_clear { None }
                else { Some(SettingValue::I32(v)) }
            },
            SettingValue::String(s) => {
                if s == value && allow_clear { None }
                else { Some(SettingValue::String(value.to_string())) }
            },
        };
        Ok(value)
    }
}
pub fn color_from_string(value: &str) -> SettingsResult<Color> {
    let values: Vec<&str> = value.split(',').collect();
    Ok(Color::from_rgba(values[0].trim().parse::<f32>()?, 
                values[1].trim().parse::<f32>()?, 
                values[2].trim().parse::<f32>()?, 
                values[3].trim().parse::<f32>()?))
}

#[repr(C)]
pub enum SelectedAction {
    Start(std::process::Command),
    Load(String),
}

#[repr(C)]
pub struct YaffePluginItem {
    pub name: String,
    pub path: String,
    pub thumbnail: PathType,
    pub restricted: bool,
    pub description: String,
}
impl YaffePluginItem {
    pub fn new(name: String, path: String, thumbnail: PathType, restricted: bool, description: String) -> YaffePluginItem {
        YaffePluginItem {
            name,
            path,
            thumbnail,
            restricted,
            description,
        }
    }
}

pub type InitializeResult = Result<(), String>;
pub type LoadResult = Result<LoadedItems, String>;

#[repr(C)]
pub struct LoadedItems {
    pub results: Vec<YaffePluginItem>,
    pub next_page: String,
}
impl LoadedItems {
    pub fn next(results: Vec<YaffePluginItem>, next_page: String) -> LoadResult {
        Ok(LoadedItems { results, next_page })
    }
    pub fn finish(results: Vec<YaffePluginItem>) -> LoadResult {
        Ok(LoadedItems { results, next_page: String::from("") })
    }
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! create_plugin {
    ($init:expr) => {
        #[no_mangle]
        pub fn initialize() -> Box<dyn YaffePlugin> {
            Box::new($init)
        }
    };
}

pub trait YaffePlugin {
    fn name(&self) -> &'static str;
    fn initialize(&mut self, settings: &HashMap<String, SettingValue>) -> InitializeResult;
    fn settings(&self) -> Vec<(&'static str, SettingValue)>;
    fn load_items(&mut self, size: u32, navigation_state: &[String], page: &str) -> LoadResult;
    fn on_selected(&mut self, name: &str, path: &str) -> SelectedAction;
}

pub fn try_get_str(settings: &HashMap<String, SettingValue>, name: &str) -> Option<String> {
    if let Some(SettingValue::String(s)) = settings.get(name) {
        return Some(s.clone());
    }
    None
}

pub fn try_get_i32(settings: &HashMap<String, SettingValue>, name: &str) -> Option<i32> {
    if let Some(SettingValue::I32(s)) = settings.get(name) {
        return Some(*s);
    }
    None
}

pub fn try_get_f32(settings: &HashMap<String, SettingValue>, name: &str) -> Option<f32> {
    if let Some(SettingValue::F32(s)) = settings.get(name) {
        return Some(*s);
    }
    None
}
