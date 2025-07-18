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

            fn get_default(value: &str) -> Option<SettingValue> {
                match value {
                    $($display => Some($default),)+
                    &_ => None,
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
        InfoScrollSpeed("info_scroll_speed") = SettingValue::F32(5.),
        MaxRows("max_rows") = SettingValue::I32(4),
        MaxColumns("max_columns") = SettingValue::I32(4),
        FontColor("font_color") = SettingValue::Tuple((0.95, 0.95, 0.95, 1.)),
        AccentColor("accent_color") = SettingValue::Tuple((0.25, 0.3, 1., 1.)),
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

            let value = if let Some(value) = self.settings.get(key) {
                value.clone()
            } else {
                SettingNames::get_default(key).unwrap()
            };

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
                SettingNames::get_default(name).unwrap()
            };

            result.push((name.to_string(), value));
        }
        result
    }

    pub fn set_setting(&mut self, name: &str, value: &str) -> Result<(), SettingLoadError> {
        assert!(SETTINGS.contains(&name));

        let setting = SettingNames::get_default(name).unwrap();
        let value = setting_from_string(name, &setting, value, true)?;
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
    settings_get!(get_tuple, (f32, f32, f32, f32), SettingValue::Tuple);

    pub fn serialize(&self) -> Result<(), std::io::Error> {
        fn write_line(name: &str, value: &SettingValue, file: &mut std::fs::File) -> Result<(), std::io::Error> {
            let line = format!("{name} = {value}\n");
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

// TODO combine with parse_value
pub fn setting_from_string(
    name: &str,
    setting: &SettingValue,
    value: &str,
    allow_clear: bool,
) -> SettingsResult<Option<SettingValue>> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    let v = parse_value(value)?;

    if std::mem::discriminant(&v) != std::mem::discriminant(setting) {
        return Err(SettingLoadError::InvalidType(name.to_string()));
    }
    if &v == setting && allow_clear {
        Ok(None)
    } else {
        Ok(Some(v))
    }
}

/// Loads settings from a file path
pub fn load_settings<P: Clone + AsRef<Path>>(path: P, validate_names: bool) -> SettingsResult<SettingsFile> {
    let mut path_buf = std::path::PathBuf::new();
    path_buf.push(path.clone());
    let last_write = std::fs::metadata(path_buf.clone())?.modified();

    let settings = SettingsFile {
        settings: load_settings_from_path(path, validate_names)?,
        path: path_buf,
        last_write: last_write.unwrap(),
    };

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
            settings.settings = load_settings_from_path(settings.path.clone(), true)?;
            return Ok(true);
        }
    }

    Ok(false)
}

/// Loads settings from a file path
pub fn load_settings_from_path<P: Clone + AsRef<Path>>(
    path: P,
    validate_names: bool,
) -> SettingsResult<HashMap<String, SettingValue>> {
    let mut path_buf = std::path::PathBuf::new();
    path_buf.push(path);
    let data = std::fs::read_to_string(path_buf.clone())?;

    let mut current_settings = HashMap::new();
    for line in data.lines() {
        //# denotes a comment
        if !line.starts_with('#') && !line.is_empty() {
            let (key, value) = line.split_at(line.find('=').ok_or(SettingLoadError::IncorrectFormat)?);

            //First character will be =, dont include that
            let key = key.trim();
            let value = value[1..].trim();
            let value = parse_value(value)?;
            if let Some(default) = SettingNames::get_default(key) {
                if validate_names && std::mem::discriminant(&default) != std::mem::discriminant(&value) {
                    return Err(SettingLoadError::InvalidType(key.to_string()));
                }
            }
            current_settings.insert(String::from(key), value);
        }
    }

    Ok(current_settings)
}

fn parse_value(value: &str) -> Result<SettingValue, SettingLoadError> {
    if value.starts_with("(") && value.ends_with(")") {
        let inner = &value[1..value.len() - 1]; // Remove parentheses
        let parts: Vec<SettingValue> = inner.split(',').map(|s| parse_value(s.trim()).unwrap()).collect();
        match parts.as_slice() {
            [SettingValue::F32(f1), SettingValue::F32(f2), SettingValue::F32(f3), SettingValue::F32(f4)] => {
                Ok(SettingValue::Tuple((*f1, *f2, *f3, *f4)))
            }
            _ => Err(SettingLoadError::IncorrectFormat),
        }
    } else if let Ok(i32) = value.parse::<i32>() {
        Ok(SettingValue::I32(i32))
    } else if let Ok(f32) = value.parse::<f32>() {
        Ok(SettingValue::F32(f32))
    } else {
        Ok(SettingValue::String(String::from(value)))
    }
}
