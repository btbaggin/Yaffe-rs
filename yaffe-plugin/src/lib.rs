use std::collections::HashMap;

pub struct YaffePluginItem {
    pub name: String,
    pub path: String,
    pub thumbnail: PathType,
    pub restricted: bool,
    pub description: String,
}

#[derive(Clone)]
pub enum PluginSetting {
    String(String),
    F32(f32),
    I32(i32),
}

pub enum PathType {
    Url(String),
    File(String),
}

pub enum SelectedAction {
    Start(std::process::Command),
    Load(String),
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
    fn initialize(&mut self, settings: &HashMap<String, PluginSetting>) -> InitializeResult;
    fn settings(&self) -> Vec<(&'static str, PluginSetting)>;
    fn load_items(&mut self, size: u32, navigation_state: &Vec<String>, page: &str) -> LoadResult;
    fn on_selected(&mut self, name: &str, path: &str) -> SelectedAction;
}

pub fn try_get_str(settings: &HashMap<String, PluginSetting>, name: &'static str) -> Option<String> {
    if let Some(value) = settings.get(name) {
        if let PluginSetting::String(s) = value {
            return Some(s.clone());
        }
    }
    None
}

pub fn try_get_i32(settings: &HashMap<String, PluginSetting>, name: &'static str) -> Option<i32> {
    if let Some(value) = settings.get(name) {
        if let PluginSetting::I32(s) = value {
            return Some(*s);
        }
    }
    None
}

pub fn try_get_f32(settings: &HashMap<String, PluginSetting>, name: &'static str) -> Option<f32> {
    if let Some(value) = settings.get(name) {
        if let PluginSetting::F32(s) = value {
            return Some(*s);
        }
    }
    None
}
