use std::{collections::HashMap, path::PathBuf};

mod settings;
pub use settings::{SettingLoadError, SettingValue, SettingsResult};

#[repr(C)]
#[derive(Debug, Clone)]
pub enum PathType {
    File(PathBuf),
    Url(PathBuf),
}

#[repr(C)]
pub enum SelectedAction {
    Process(std::process::Command),
    Webview(String),
}

#[repr(C)]
pub enum TileType {
    Folder,
    App,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginFilter {
    pub name: String,
    pub options: Vec<String>,
}
impl PluginFilter {
    pub fn new(name: &str, options: &[&str]) -> PluginFilter {
        PluginFilter { name: name.to_string(), options: options.iter().map(|o| o.to_string()).collect() }
    }

    pub fn from_range(name: &str, start: &str, end: &str) -> PluginFilter {
        PluginFilter { name: name.to_string(), options: Self::generate_string_range(start, end) }
    }

    fn generate_string_range(start: &str, end: &str) -> Vec<String> {
        let start_ascii = start.chars().next().unwrap() as u8;
        let end_ascii = end.chars().next().unwrap() as u8;

        (start_ascii..=end_ascii).map(|c| String::from_utf8(vec![c]).unwrap_or_default()).collect()
    }
}

#[repr(C)]
pub struct YaffePluginItem {
    pub name: String,
    pub path: String,
    pub tile_type: TileType,
    pub description: String,
    pub thumbnail: PathType,
    pub restricted: bool,
    pub metadata: HashMap<String, String>,
}
impl YaffePluginItem {
    pub fn folder(
        name: String,
        path: String,
        thumbnail: PathType,
        restricted: bool,
        description: String,
        metadata: HashMap<String, String>,
    ) -> YaffePluginItem {
        YaffePluginItem { name, path, tile_type: TileType::Folder, description, thumbnail, restricted, metadata }
    }

    pub fn app(
        name: String,
        path: String,
        thumbnail: PathType,
        restricted: bool,
        description: String,
        metadata: HashMap<String, String>,
    ) -> YaffePluginItem {
        YaffePluginItem { name, path, tile_type: TileType::App, description, thumbnail, restricted, metadata }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TileQuery {
    pub filter: Option<String>,
    pub value: Option<String>,
    pub limit: usize,
}

pub type InitializeResult = Result<Vec<PluginFilter>, String>;
pub type LoadResult = Result<Vec<YaffePluginItem>, String>;

#[macro_export]
#[allow(unused_macros)]
macro_rules! create_plugin {
    ($init:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn create_plugin() -> Box<dyn YaffePlugin> { Box::new($init) }
    };
}

pub trait YaffePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&mut self, settings: &HashMap<String, SettingValue>) -> InitializeResult;
    fn load_tiles(&mut self, query: &TileQuery) -> LoadResult;
    fn select_tile(&self, name: &str, path: &str, tile_type: &TileType) -> SelectedAction;
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
