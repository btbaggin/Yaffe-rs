use yaffe_plugin::YaffePlugin;
use dlopen::wrapper::{Container, WrapperApi};
use crate::logger::{LogEntry, UserMessage};
use crate::settings::SettingValue;
use std::collections::HashMap;
pub use yaffe_plugin::*;

pub struct Plugin {
	_container: Container<PluginWrapper>, //There for keeping reference to the library
	pub settings: HashMap<String, PluginSetting>,
	data: Box<dyn YaffePlugin>,
}
impl Plugin {
	//We cant impl Deref because we need to own the settings.
	//We will get borrow settings when we try to call a mut method passing in out immut settings
	pub fn name(&self) -> &'static str { self.data.name() }
    pub fn initialize(&mut self) -> InitializeResult { self.data.initialize(&self.settings) }
    pub fn load_items(&mut self, initial: bool) -> LoadResult { self.data.load_items(initial, &self.settings) }
	pub fn start(&self, name: &str, path: &str) -> std::process::Command { self.data.start(name, path, &self.settings) }
}

#[derive(WrapperApi)]
struct PluginWrapper {
	initialize: fn() -> Box<dyn yaffe_plugin::YaffePlugin>,
}

pub fn load_plugins(state: &mut crate::YaffeState, directory: &str, mut plugins: crate::settings::PluginSettings) {
	let path = std::fs::canonicalize(directory).unwrap();

	for entry in std::fs::read_dir(path).log_if_fail() {
		let path = entry.unwrap().path();

		if let Some(ext) = path.extension() {
			let ext = ext.to_string_lossy();

			#[cfg(windows)] 
			let ok = ext == "dll";
			#[cfg(not(windows))] 
			let ok = ext == "so";

			if ok && path.is_file() {
				let file = path.file_stem().unwrap().to_string_lossy();
				let message = format!("Failed to load plugin {:?}", file);

				let container: Option<Container<PluginWrapper>> = unsafe { Container::load(path.clone()) }.display_failure(&message, state);
				if let Some(cont) = container {
					//Create our YaffePlugin object
					let data = cont.initialize();
					
					//Get an owned reference to our settings
					let settings = if let Some(settings) = plugins.remove(&file.to_string()) { settings } 
								   else { HashMap::new() };

					//Do any initialization work on the object
					let mut plugin = Plugin { 
						_container: cont, 
						settings: translate_to_plugin_settings(settings),
						data 
					};

					if plugin.data.initialize(&plugin.settings).display_failure(&message, state).is_some() {
						state.plugins.push(std::cell::RefCell::new(plugin));
					}
				}
			}
		}
	}
}

pub fn unload(plugins: &mut Vec<std::cell::RefCell<Plugin>>) {
	//TODO this crashes things
	plugins.clear();
}

fn translate_to_plugin_settings(settings: HashMap<String, SettingValue>) -> HashMap<String, PluginSetting> {
	let mut result = HashMap::new();
	for (key, value) in settings.iter() {

		let value = match value {
			SettingValue::F32(f) => Some(PluginSetting::F32(*f)),
			SettingValue::I32(i) => Some(PluginSetting::I32(*i)),
			SettingValue::String(s) => Some(PluginSetting::String(s.clone())),
			SettingValue::Color(_) => None
		};
		if let Some(value) = value {
			result.insert(key.clone(), value);
		}
	}
	result
}