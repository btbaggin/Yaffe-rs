use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use speedy2d::color::Color;
use crate::logger::{LogTypes, log_entry};
use std::convert::AsRef;

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

#[derive(Clone)]
pub enum SettingValue {
    String(String),
    F32(f32),
    I32(i32),
    Color(Color),
}

macro_rules! stringy_enum {
    (pub enum $name:ident {
        $($value:ident($display:expr) = $default:expr,)+
    }) => {
        #[derive(Copy, Clone)]
        pub enum $name {
            $($value,)+
        } 

        impl $name {
            pub fn to_string(value: $name) -> &'static str {
                match value {
                    $($name::$value => $display,)+
                }
            }

            pub fn get_default(value: $name) -> SettingValue {
                match value {
                    $($name::$value => $default,)+
                }
            }
        }
    };
}

stringy_enum! {
    pub enum SettingNames {
        InfoFontSize("info_font_size") = SettingValue::F32(18.),
        TitleFontSize("title_font_size") = SettingValue::F32(32.),
        LightShadeFactor("light_shade_factor") = SettingValue::F32(0.3),
        DarkShadeFactor("dark_shade_factor") = SettingValue::F32(-0.6),
        InfoScrollSpeed("info_scroll_speed") = SettingValue::F32(20.),
        RestrictedApprovalValidTime("restricted_approval_valid_time") = SettingValue::I32(3600),
        ItemsPerRow("items_per_row") = SettingValue::I32(4),
        ItemsPerColumn("items_per_column") = SettingValue::I32(3),
        FontColor("font_color") = SettingValue::Color(Color::from_rgba(0.95, 0.95, 0.95, 1.)),
        AccentColor("accent_color") = SettingValue::Color(Color::from_rgba(0.25, 0.3, 1., 1.)),
        RecentPageCount("recent_page_count") = SettingValue::F32(1.),
    }
}

macro_rules! settings_get {
    ($name:ident, $ty:ty, $setting:path) => {
        #[allow(dead_code)]
        pub fn $name(&self, setting: crate::settings::SettingNames) -> $ty {
            let key = crate::settings::SettingNames::to_string(setting);
            if let Some(f) = self.settings.get(key) {
                if let $setting(value) = f {
                    return value.clone();
                }
                log_entry(LogTypes::Warning, format!("Attemted to retrieve setting {} but type expected type {}", key, stringify!($ty)));
            }
            if let $setting(value) = crate::settings::SettingNames::get_default(setting) {
                return value;
            }
            panic!("Accessed setting using incorrect type");
        }
    };
}

type SettingsResult<T> = Result<T, SettingLoadError>;
#[derive(Clone)]
pub struct SettingsFile {
    settings: HashMap<String, SettingValue>,
    path: std::path::PathBuf,
    last_write: SystemTime
}
impl SettingsFile {
    pub fn default() -> SettingsFile {
        SettingsFile { settings: HashMap::default(), path: std::path::PathBuf::default(), last_write: SystemTime::now() }
    }

    settings_get!(get_f32, f32, SettingValue::F32);
    settings_get!(get_i32, i32, SettingValue::I32);
    settings_get!(get_str, String, SettingValue::String);
    settings_get!(get_color, Color, SettingValue::Color);
}

/// Loads settings from a file path
pub fn load_settings<P: Clone + AsRef<Path>>(path: P) -> SettingsResult<SettingsFile> {
    let mut path_buf = std::path::PathBuf::new(); path_buf.push(path);
    let data = std::fs::read_to_string(path_buf.clone())?;
    let last_write = std::fs::metadata(path_buf.clone())?.modified();
    let mut settings = SettingsFile { settings: HashMap::new(), path: path_buf, last_write: last_write.unwrap() };
    
    populate_settings(&mut settings, data)?;

    Ok(settings)
}

/// Checks for and loads any updates to the settings file
pub fn update_settings(settings: &mut SettingsFile) -> SettingsResult<()> {
    //We log an error if the file isnt found in load_settings
    //Since this is already logged we dont need to get logging it every frame
    if settings.path.as_path().exists() {
        let last_write = std::fs::metadata(settings.path.clone())?.modified()?;

        if last_write > settings.last_write {
            let data = std::fs::read_to_string(settings.path.clone())?;
            settings.last_write = last_write;

            settings.settings.clear();
            return populate_settings(settings, data);
        }
    }

    Ok(())
}

fn populate_settings(settings: &mut SettingsFile, data: String) -> SettingsResult<()> {
    for line in data.lines() {
        //# denotes a comment
        if !line.starts_with('#') && !line.is_empty() {

            let (key, type_value) = line.split_at(line.find(':').ok_or(SettingLoadError::IncorrectFormat)?);
            let (ty, value) = type_value.split_at(type_value.find('=').ok_or(SettingLoadError::IncorrectFormat)?);

            //First character will be : or =, dont include that
            let value = value[1..].trim();
            let value = match ty[1..].trim() {
                "f32" => SettingValue::F32(value.parse::<f32>()?),
                "i32" => SettingValue::I32(value.parse::<i32>()?),
                "str" => SettingValue::String(String::from(value)),
                "color" => SettingValue::Color(color_from_string(value)?),
                _ => return Err(SettingLoadError::InvalidType),
            };
            settings.settings.insert(String::from(key.trim()), value);
        }
    }

    Ok(())
}

fn color_from_string(value: &str) -> SettingsResult<Color> {
    let values: Vec<&str> = value.split(',').collect();
    Ok(Color::from_rgba(values[0].trim().parse::<f32>()?, 
                values[1].trim().parse::<f32>()?, 
                values[2].trim().parse::<f32>()?, 
                values[3].trim().parse::<f32>()?))
}