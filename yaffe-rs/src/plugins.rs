use yaffe_plugin::YaffePlugin;
use std::ops::{Deref, DerefMut};
use dlopen::wrapper::{Container, WrapperApi};
use crate::logger::LogEntry;
pub use yaffe_plugin::{YaffePluginItem};

pub struct Plugin {
	_container: Container<PluginWrapper>, //There for keeping reference to the library
	data: Box<dyn YaffePlugin>,
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

pub fn load_plugins(plugins: &mut Vec<Plugin>, directory: &str) {
	let path = std::fs::canonicalize(directory).unwrap();

	for entry in std::fs::read_dir(path).log_if_fail() {
		let path = entry.unwrap().path();

		if let Some(ext) = path.extension() {
			let ext = ext.to_string_lossy();

			let mut ok = false;
			#[cfg(windows)]
			if ext == "dll" { ok = true } 
			#[cfg(not(windows))]
			if ext == "so" { ok = true }

			if ok && path.is_file() {
				let container: Container<PluginWrapper> = unsafe { Container::load(path) }.expect("Something bad");
				let data = container.initialize();

				let mut plugin = Plugin { _container: container, data };
				if let Err(s) = plugin.data.initialize() {
					//TODO display user error
					crate::logger::log_entry_with_message(crate::logger::LogTypes::Error, s, "Unable to load plugin");
				} else {
					plugins.push(plugin);
				}
			}
		}
	}
}

pub fn unload(plugins: &mut Vec<Plugin>) {
	//TODO this crashes things
	plugins.clear();
}