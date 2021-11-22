use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use speedy2d::color::Color;
use std::convert::{AsRef, TryFrom};
use std::io::Write;
use crate::colors::rgba_string;

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

pub type PluginSettings = HashMap<String, HashMap<String, SettingValue>>;

#[derive(Clone)]
pub enum SettingValue {
    String(String),
    F32(f32),
    I32(i32),
    Color(Color),
}
impl From<yaffe_plugin::PluginSetting> for SettingValue {
    fn from(setting: yaffe_plugin::PluginSetting) -> Self {
        match setting {
            yaffe_plugin::PluginSetting::F32(f) => SettingValue::F32(f),
            yaffe_plugin::PluginSetting::I32(i) => SettingValue::I32(i),
            yaffe_plugin::PluginSetting::String(s) => SettingValue::String(s),
        }
    }
}
impl TryFrom<&SettingValue> for yaffe_plugin::PluginSetting {
    type Error = &'static str;
    fn try_from(setting: &SettingValue) -> Result<Self, Self::Error> {
        match setting {
            SettingValue::F32(f) => Ok(yaffe_plugin::PluginSetting::F32(*f)),
            SettingValue::I32(i) => Ok(yaffe_plugin::PluginSetting::I32(*i)),
            SettingValue::String(s) => Ok(yaffe_plugin::PluginSetting::String(s.clone())),
            SettingValue::Color(_) => Err("Invalid plugin setting"),
        }
    }
}

impl SettingValue {
    fn from_string(&self, value: &str, allow_clear: bool) -> Result<Option<SettingValue>, SettingLoadError> {
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

macro_rules! stringy_enum {
    (pub enum $name:ident {
        $($value:ident($display:expr) = $default:expr,)+
    }) => {
        #[derive(Copy, Clone)]
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
        AssetCacheSizeMb("asset_cache_size_mb") = SettingValue::I32(32),
    }
}

macro_rules! settings_get {
    ($name:ident, $ty:ty, $setting:path) => {
        #[allow(dead_code)]
        pub fn $name(&self, setting: crate::settings::SettingNames) -> $ty {
            let key = crate::settings::SettingNames::to_string(setting);

            let value = if let Some(value) = self.settings.get(key) { value.clone() }
            else { SettingNames::get_default(key) };

            if let $setting(value) = value {
                return value.clone();
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
    last_write: SystemTime,
    plugins: PluginSettings,
}
impl SettingsFile {
    pub fn default() -> SettingsFile {
        SettingsFile { 
            settings: HashMap::default(), 
            path: std::path::PathBuf::default(), 
            last_write: SystemTime::now(),
            plugins: PluginSettings::default(),
        }
    }

    pub fn plugin(&self, name: &str) -> HashMap<String, yaffe_plugin::PluginSetting> {
        let settings = self.plugins.get(name).unwrap();
        translate_to_plugin_settings(settings)
    }

    /// Ensures the plugin settings have all possible values.
    /// Should only be called on plugin initialization
    pub fn populate_plugin_settings(&mut self, plugin: &crate::plugins::Plugin) {
        let settings = self.plugins.entry(plugin.file.clone()).or_insert(HashMap::default());

        for (name, default) in plugin.settings() {
            //Add any missing settings
            if !settings.contains_key(name) { 
                settings.insert(name.to_string(), default.into());
            } 
        }
    }

    /// Returns all possible settings that can be set and their current (or default) values
    pub fn get_full_settings(&self, plugin: Option<&str>) -> Vec<(String, SettingValue)> {
        let mut result = vec!();
        match plugin {
            Some(plugin) => {
                let settings = self.plugins.get(plugin).unwrap();

                for (name, default) in settings {
                    result.push((name.clone(), default.clone()))
                }
            }
            None => {
                for name in SETTINGS {
                    //Get configured value if it exists, otherwise default
                    let value = if let Some(value) = self.settings.get(*name) { value.clone() }
                    else { SettingNames::get_default(name) };

                    result.push((name.to_string(), value));
                }
            }
        };
        result
    }

    pub fn set_setting(&mut self, plugin: Option<&String>, name: &str, value: &str) -> Result<(), SettingLoadError> {
        let (settings, value) = match plugin {
            Some(file) => {
                //It's ok to do unwrap here because they are gauranteed to be present due to populate_plugin_settings
                let settings = self.plugins.get_mut(file).unwrap();
                let setting = settings.get(name).unwrap().clone();

                (settings, setting.from_string(value, false)?)
            },
            None => {
                assert!(SETTINGS.iter().position(|&n| n == name).is_some());

                let setting = SettingNames::get_default(name);
                (&mut self.settings, setting.from_string(value, true)?)
            },
        };
        
        //Value was either removed or the default, don't add it
        if value.is_none() { 
            settings.remove(name);
            return Ok(()); 
        }

        //Add or insert new value
        let value = value.unwrap();
        if let None = settings.get(name) {
            settings.insert(name.to_string(), value);
        } else {
            *settings.get_mut(name).unwrap() = value;
        }
        Ok(())
    }

    settings_get!(get_f32, f32, SettingValue::F32);
    settings_get!(get_i32, i32, SettingValue::I32);
    settings_get!(get_str, String, SettingValue::String);
    settings_get!(get_color, Color, SettingValue::Color);

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

        //write settings for each plugin
        for (plugin, settings) in self.plugins.iter() {
            file.write_all(format!("\n--{}\n", plugin).as_bytes())?;

            for (key, value) in settings.iter() {
                write_line(key, value, &mut file)?;
            }
        }

        Ok(())
    }
}

fn translate_to_plugin_settings(settings: &HashMap<String, SettingValue>) -> HashMap<String, yaffe_plugin::PluginSetting> {
	let mut result = HashMap::new();
	for (key, value) in settings.iter() {

		if let Ok(value) = yaffe_plugin::PluginSetting::try_from(value) {
			result.insert(key.clone(), value);
		}
	}
	result
}

/// Loads settings from a file path
pub fn load_settings<P: Clone + AsRef<Path>>(path: P) -> SettingsResult<SettingsFile> {
    let mut path_buf = std::path::PathBuf::new(); path_buf.push(path);
    let data = std::fs::read_to_string(path_buf.clone())?;
    let last_write = std::fs::metadata(path_buf.clone())?.modified();
    
    let mut settings = SettingsFile { 
        settings: HashMap::new(), 
        path: path_buf, 
        last_write: last_write.unwrap(), 
        plugins: PluginSettings::new(),
    };
    
    populate_settings(&mut settings, data)?;

    Ok(settings)
}

/// Checks for and loads any updates to the settings file
pub fn update_settings(settings: &mut SettingsFile) -> SettingsResult<bool> {
    //We log an error if the file isnt found in load_settings
    //Since this is already logged we dont need to get logging it every frame
    if settings.path.as_path().exists() {
        let last_write = std::fs::metadata(settings.path.clone())?.modified()?;

        if last_write > settings.last_write {
            let data = std::fs::read_to_string(settings.path.clone())?;
            settings.last_write = last_write;

            settings.settings.clear();
            populate_settings(settings, data)?;
            return Ok(true);
        }
    }

    Ok(false)
}

fn populate_settings(settings: &mut SettingsFile, data: String) -> SettingsResult<()> {
    let mut current_settings = &mut settings.settings;
    for line in data.lines() {

        if line.starts_with("--") {
            let (_, name) = line.split_at(2);
            settings.plugins.insert(String::from(name), HashMap::default());
            current_settings = settings.plugins.get_mut(name).unwrap();

        }
        //# denotes a comment
        else if !line.starts_with('#') && !line.is_empty() {

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

    Ok(())
}

fn color_from_string(value: &str) -> SettingsResult<Color> {
    let values: Vec<&str> = value.split(',').collect();
    Ok(Color::from_rgba(values[0].trim().parse::<f32>()?, 
                values[1].trim().parse::<f32>()?, 
                values[2].trim().parse::<f32>()?, 
                values[3].trim().parse::<f32>()?))
}