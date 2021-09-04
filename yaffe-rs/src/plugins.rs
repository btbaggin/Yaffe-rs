use yaffe_plugin::YaffePlugin;
use dlopen::wrapper::{Container, WrapperApi};
use crate::logger::{LogEntry, UserMessage};
pub use yaffe_plugin::*;
use std::ops::{DerefMut, Deref};
use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum PluginLoadType {
	Initial,
	Refresh,
	Append,
}

pub struct Plugin {
	_container: Container<PluginWrapper>, //There for keeping reference to the library
	pub file: String,
	pub page: yaffe_plugin::LoadStatus,
	data: Box<dyn YaffePlugin>,
}
impl Plugin {
	pub fn load(&mut self, kind: PluginLoadType, size: u32, settings: &HashMap<String, PluginSetting>) -> Result<Vec<yaffe_plugin::YaffePluginItem>, String> {
		match kind {
			PluginLoadType::Initial => {
				self.data.initial_load();
				self.page = yaffe_plugin::LoadStatus::Initial;
			},
			PluginLoadType::Refresh => {
				self.page = yaffe_plugin::LoadStatus::Initial;
			},
			PluginLoadType::Append => {},
		}

		if Plugin::needs_load(self.page) {
			let result = self.data.load_items(size, settings);
			match result {
				Ok((items, page)) => {
					self.page = yaffe_plugin::LoadStatus::Refresh(page);
					return Ok(items)
				}
				Err(s) => Err(s)
			}
		} else
		{
			Ok(vec!())
		}
	}

	fn needs_load(page: LoadStatus) -> bool {
		match page {
			LoadStatus::Initial => true,
			LoadStatus::Refresh(p) => p,
		}
	}
}

impl Deref for Plugin {
    type Target = Box<dyn YaffePlugin>;

    fn deref(&self) -> &Box<dyn YaffePlugin> {
        &self.data
    }
}
impl DerefMut for Plugin {
    fn deref_mut(&mut self) -> &mut Box<dyn YaffePlugin> {
        &mut self.data
    }
}

#[derive(WrapperApi)]
struct PluginWrapper {
	initialize: fn() -> Box<dyn yaffe_plugin::YaffePlugin>,
}

pub fn load_plugins(state: &mut crate::YaffeState, directory: &str) {
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
					
					//Do any initialization work on the object
					let mut plugin = Plugin { 
						_container: cont, 
						file: file.to_string(),
						page: yaffe_plugin::LoadStatus::Initial,
						data 
					};

					let settings = state.settings.plugin(&plugin.file);
					if plugin.data.initialize(&settings).display_failure(&message, state).is_some() {
						state.plugins.push(std::cell::RefCell::new(plugin));
					}
				}
			}
		}
	}
}

pub fn load_plugin_items(kind: PluginLoadType, state: &mut crate::YaffeState) {
	if let Some((plugin, settings)) = state.get_platform().get_plugin(state) {
		let x = state.settings.get_i32(crate::SettingNames::ItemsPerRow);
		let y = state.settings.get_i32(crate::SettingNames::ItemsPerColumn);

		let items = plugin.borrow_mut().load(kind, (x * y) as u32, &settings);
		if let Some(items) = items.display_failure("Error loading plugin", state) {
			let platform = &mut state.platforms[state.selected_platform];
			match kind {
				PluginLoadType::Initial | PluginLoadType::Refresh => platform.apps.clear(),
				_ => {},
			}
			for i in items {
				platform.apps.push(crate::Executable::plugin_item(state.selected_platform, i));
			}
		}
	}
}

pub fn unload(plugins: &mut Vec<std::cell::RefCell<Plugin>>) {
	//TODO this crashes things
	plugins.clear();
}