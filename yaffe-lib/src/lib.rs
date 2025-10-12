use std::{collections::HashMap, path::PathBuf};

mod errors;
mod plugin_macro;
mod settings;
pub use errors::*;
pub use settings::{PluginSettings, SettingLoadError, SettingValue, SettingsResult};

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

pub enum LoadItems {
    More(Vec<PluginTile>),
    Done(Vec<PluginTile>),
}

#[repr(C)]
#[derive(Hash)]
pub struct NavigationEntry {
    pub path: String,
    pub display: String,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PluginFilter {
    pub name: String,
    pub options: Vec<String>,
    pub allow_all: bool,
}
impl PluginFilter {
    pub fn new(name: &str, options: &[&str]) -> PluginFilter {
        PluginFilter { name: name.to_string(), options: options.iter().map(|o| o.to_string()).collect(), allow_all: false }
    }

    pub fn from_range(name: &str, start: &str, end: &str) -> PluginFilter {
        PluginFilter { name: name.to_string(), options: Self::generate_string_range(start, end), allow_all: false }
    }

    pub fn search() -> PluginFilter {
        let mut options = Self::generate_string_range("A", "Z");
        options.push(String::from("Go"));
        PluginFilter { name: String::from("Search"), options, allow_all: true }
    }

    fn generate_string_range(start: &str, end: &str) -> Vec<String> {
        let start_ascii = start.chars().next().unwrap() as u8;
        let end_ascii = end.chars().next().unwrap() as u8;

        (start_ascii..=end_ascii).map(|c| String::from_utf8(vec![c]).unwrap_or_default()).collect()
    }
}

#[repr(C)]
pub struct PluginTile {
    pub name: String,
    pub path: String,
    pub tile_type: TileType,
    pub description: String,
    pub thumbnail: PathType,
    pub restricted: bool,
    pub metadata: HashMap<String, String>,
}
impl PluginTile {
    pub fn folder(
        name: String,
        path: String,
        thumbnail: PathType,
        restricted: bool,
        description: String,
        metadata: HashMap<String, String>,
    ) -> PluginTile {
        PluginTile { name, path, tile_type: TileType::Folder, description, thumbnail, restricted, metadata }
    }

    pub fn app(
        name: String,
        path: String,
        thumbnail: PathType,
        restricted: bool,
        description: String,
        metadata: HashMap<String, String>,
    ) -> PluginTile {
        PluginTile { name, path, tile_type: TileType::App, description, thumbnail, restricted, metadata }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct TileQuery {
    pub filter: Option<String>,
    pub value: Option<String>,
    pub limit: usize,
}

pub type InitializeResult = Result<(), PluginError>;
pub type LoadResult = Result<LoadItems, PluginError>;

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
    fn filters(&self) -> Vec<PluginFilter>;
    fn load_tiles(&mut self, query: &TileQuery, parent: &[NavigationEntry]) -> LoadResult;
    fn select_tile(&self, name: &str, path: &str) -> SelectedAction;
}
