use yaffe_plugin::YaffePlugin;
use std::ops::{Deref, DerefMut};
use dlopen::wrapper::{Container, WrapperApi};
use crate::logger::LogEntry;
pub use yaffe_plugin::{YaffePluginItem};
use crate::logger::UserMessage;

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
				let file = path.file_name().unwrap().to_string_lossy();
				let message = format!("Failed to load plugin {:?}", file);

				let container: Option<Container<PluginWrapper>> = unsafe { Container::load(path) }.display_failure(&message, state);
				if let Some(cont) = container {
					//Create our YaffePlugin object
					let data = cont.initialize();
					
					//Do any initialization work on the object
					let mut plugin = Plugin { _container: cont, data };
					if plugin.data.initialize().display_failure(&message, state).is_some() {
						state.plugins.push(plugin);
					}
				}
			}
		}
	}
}

pub fn unload(plugins: &mut Vec<Plugin>) {
	//TODO this crashes things
	plugins.clear();
}