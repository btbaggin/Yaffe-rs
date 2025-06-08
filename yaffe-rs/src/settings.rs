use crate::ui::rgba_string;
use std::collections::HashMap;
use std::convert::AsRef;
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
pub use yaffe_lib::{SettingLoadError, SettingValue, SettingsResult};

macro_rules! stringy_enum {
    (pub enum $name:ident {
        $($value:ident($display:expr) = $default:expr,)+
    }) => {
        #[derive(Copy, Clone, Hash, PartialEq, Eq)]
        pub enum $name {
            $($value,)+
        }
        const SETTINGS: &[&str] = &[$($display,)+];

        impl $name {
            fn to_string(value: $name) -> &'static str {
                match value {
                    $($name::$value => $display,)+
                }
            }

            fn get_default(value: &str) -> SettingValue {
                match value {
                    $($display => $default,)+
                    &_ => panic!("cant happen"),
                }
            }
        }
    };
}

stringy_enum! {
    pub enum SettingNames {
        InfoFontSize("info_font_size") = SettingValue::F32(24.),
        LightShadeFactor("light_shade_factor") = SettingValue::F32(0.3),
        DarkShadeFactor("dark_shade_factor") = SettingValue::F32(-0.6),
        InfoScrollSpeed("info_scroll_speed") = SettingValue::F32(20.),
        MaxRows("max_rows") = SettingValue::I32(4),
        MaxColumns("max_columns") = SettingValue::I32(4),
        FontColor("font_color") = SettingValue::Color((0.95, 0.95, 0.95, 1.)),
        AccentColor("accent_color") = SettingValue::Color((0.25, 0.3, 1., 1.)),
        RecentPageCount("recent_page_count") = SettingValue::F32(1.),
        AssetCacheSizeMb("asset_cache_size_mb") = SettingValue::I32(64),
        LoggingLevel("logging_level") = SettingValue::String(String::from("Info")),
    }
}

macro_rules! settings_get {
    ($name:ident, $ty:ty, $setting:path) => {
        #[allow(dead_code)]
        pub fn $name(&self, setting: crate::settings::SettingNames) -> $ty {
            let key = crate::settings::SettingNames::to_string(setting);

            let value =
                if let Some(value) = self.settings.get(key) { value.clone() } else { SettingNames::get_default(key) };

            if let $setting(value) = value {
                return value.clone();
            }
            panic!("Accessed setting using incorrect type");
        }
    };
}

#[derive(Clone)]
pub struct SettingsFile {
    settings: HashMap<String, SettingValue>,
    path: std::path::PathBuf,
    last_write: SystemTime,
}
impl SettingsFile {
    pub fn default() -> SettingsFile {
        SettingsFile {
            settings: HashMap::default(),
            path: std::path::PathBuf::default(),
            last_write: SystemTime::now(),
        }
    }

    /// Returns all possible settings that can be set and their current (or default) values
    pub fn get_full_settings(&self) -> Vec<(String, SettingValue)> {
        let mut result = vec![];
        for name in SETTINGS {
            //Get configured value if it exists, otherwise default
            let value = if let Some(value) = self.settings.get(*name) {
                value.clone()
            } else {
                SettingNames::get_default(name)
            };

            result.push((name.to_string(), value));
        }
        result
    }

    pub fn set_setting(&mut self, name: &str, value: &str) -> Result<(), SettingLoadError> {
        assert!(SETTINGS.contains(&name));

        let setting = SettingNames::get_default(name);
        let value = setting_from_string(&setting, value, true)?;
        match value {
            //Add or insert new value
            Some(v) => {
                self.settings.entry(name.to_string()).and_modify(|e| *e = v.clone()).or_insert(v);
            }
            //Value was either removed or the default, don't add it
            None => {
                self.settings.remove(name);
            }
        }
        Ok(())
    }

    settings_get!(get_f32, f32, SettingValue::F32);
    settings_get!(get_i32, i32, SettingValue::I32);
    settings_get!(get_str, String, SettingValue::String);
    settings_get!(get_color, (f32, f32, f32, f32), SettingValue::Color);

    pub fn serialize(&self) -> Result<(), std::io::Error> {
        fn write_line(name: &str, value: &SettingValue, file: &mut std::fs::File) -> Result<(), std::io::Error> {
            let line = match value {
                SettingValue::String(s) => format!("{}: {} = {}\n", name, "str", s),
                SettingValue::I32(i) => format!("{}: {} = {}\n", name, "i32", i),
                SettingValue::F32(f) => format!("{}: {} = {}\n", name, "f32", f),
                SettingValue::Color(c) => format!("{}: {} = {}\n", name, "color", rgba_string(c)),
            };

            file.write_all(line.as_bytes())
        }

        //write base settings
        let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open(self.path.clone())?;
        for (key, value) in self.settings.iter() {
            write_line(key, value, &mut file)?;
        }

        Ok(())
    }
}

pub fn setting_from_string(
    setting: &SettingValue,
    value: &str,
    allow_clear: bool,
) -> SettingsResult<Option<SettingValue>> {
    if value.is_empty() {
        return Ok(None);
    }

    let value = match setting {
        SettingValue::Color(c) => {
            let v = color_from_string(value)?;
            if &v == c && allow_clear {
                None
            } else {
                Some(SettingValue::Color(v))
            }
        }
        SettingValue::F32(f) => {
            let v = value.parse::<f32>()?;
            if &v == f && allow_clear {
                None
            } else {
                Some(SettingValue::F32(v))
            }
        }
        SettingValue::I32(i) => {
            let v = value.parse::<i32>()?;
            if &v == i && allow_clear {
                None
            } else {
                Some(SettingValue::I32(v))
            }
        }
        SettingValue::String(s) => {
            if s == value && allow_clear {
                None
            } else {
                Some(SettingValue::String(value.to_string()))
            }
        }
    };
    Ok(value)
}

/// Loads settings from a file path
pub fn load_settings<P: Clone + AsRef<Path>>(path: P) -> SettingsResult<SettingsFile> {
    let mut path_buf = std::path::PathBuf::new();
    path_buf.push(path.clone());
    let last_write = std::fs::metadata(path_buf.clone())?.modified();

    let settings =
        SettingsFile { settings: load_settings_from_path(path)?, path: path_buf, last_write: last_write.unwrap() };

    Ok(settings)
}

/// Checks for and loads any updates to the settings file
pub fn update_settings(settings: &mut SettingsFile) -> SettingsResult<bool> {
    //We log an error if the file isnt found in load_settings
    //Since this is already logged we dont need to get logging it every frame
    if settings.path.as_path().exists() {
        let last_write = std::fs::metadata(settings.path.clone())?.modified()?;

        if last_write > settings.last_write {
            settings.last_write = last_write;
            settings.settings = load_settings_from_path(settings.path.clone())?;
            return Ok(true);
        }
    }

    Ok(false)
}

/// Loads settings from a file path
pub fn load_settings_from_path<P: Clone + AsRef<Path>>(path: P) -> SettingsResult<HashMap<String, SettingValue>> {
    let mut path_buf = std::path::PathBuf::new();
    path_buf.push(path);
    let data = std::fs::read_to_string(path_buf.clone())?;

    let mut current_settings = HashMap::new();
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
            current_settings.insert(String::from(key.trim()), value);
        }
    }

    Ok(current_settings)
}

pub fn color_from_string(value: &str) -> SettingsResult<(f32, f32, f32, f32)> {
    let values: Vec<&str> = value.split(',').collect();
    Ok((
        values[0].trim().parse::<f32>()?,
        values[1].trim().parse::<f32>()?,
        values[2].trim().parse::<f32>()?,
        values[3].trim().parse::<f32>()?,
    ))
}
