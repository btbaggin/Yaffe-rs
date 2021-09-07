// #![no_std]
// use core::panic::PanicInfo;

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

#[derive(Copy, Clone)]
pub enum LoadStatus {
    Initial,
    Refresh(Page)
}

pub enum SelectedAction {
    Start(std::process::Command),
    Load,
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
pub type LoadResult = Result<(Vec<YaffePluginItem>, Page), String>;
pub type Page = bool;

pub trait YaffePlugin {
    fn name(&self) -> &'static str;
    fn initialize(&mut self, settings: &HashMap<String, PluginSetting>) -> InitializeResult;
    fn initial_load(&mut self);
    fn load_items(&mut self, size: u32, settings: &HashMap<String, PluginSetting>) -> LoadResult;
    fn on_selected(&mut self, name: &str, path: &str, settings: &HashMap<String, PluginSetting>) -> SelectedAction;
    fn on_back(&mut self) -> bool { false }
}

pub fn load_next_page(results: Vec<YaffePluginItem>) -> LoadResult {
    Ok((results, true))
}

pub fn finish_loading(results: Vec<YaffePluginItem>) -> LoadResult {
    Ok((results, false))
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


// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

