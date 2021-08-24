// #![no_std]
// use core::panic::PanicInfo;

pub struct YaffePluginItem {
    pub name: String,
    pub path: String,
    pub thumbnail: String,
    pub restricted: bool,
    //TODO description
}
  
impl YaffePluginItem {
    pub fn new(name: String, path: String, thumbnail: String, restricted: bool) -> YaffePluginItem {
        YaffePluginItem {
            name,
            path,
            thumbnail,
            restricted,
        }
    }
}

pub type InitializeResult = Result<(), String>;
pub type LoadResult = Result<Vec<YaffePluginItem>, String>;

pub trait YaffePlugin {
    fn name(&self) -> &'static str;
    fn initialize(&mut self) -> InitializeResult;
    fn load_items(&mut self, initial: bool) -> LoadResult;
    fn start(&self, name: &str, path: &str) -> std::process::Command;
}



// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }

