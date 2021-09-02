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
pub type LoadResult = Result<Vec<YaffePluginItem>, String>;

pub trait YaffePlugin {
    fn name(&self) -> &'static str;
    fn initialize(&mut self, settings: &HashMap<String, PluginSetting>) -> InitializeResult;
    fn load_items(&mut self, initial: bool, settings: &HashMap<String, PluginSetting>) -> LoadResult;
    fn start(&self, name: &str, path: &str, settings: &HashMap<String, PluginSetting>) -> std::process::Command;
}



// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

