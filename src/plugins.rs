use yaffe_plugin::{YaffePlugin, YaffePluginItem, SelectedAction};
use dlopen::wrapper::{Container, WrapperApi};
use crate::logger::{PanicLogEntry, UserMessage, info};
use std::ops::{DerefMut, Deref};

#[derive(Copy, Clone)]
pub enum NavigationAction {
	Initialize,
	Refresh,
	Fetch,
	Back,
}

pub struct Plugin {
	//Because _container is a library that loaded data, data must be dropped before _container
	//This is done by placing data before _container in this struct
	//This must not change
	data: Box<dyn YaffePlugin>,
	_container: Container<PluginWrapper>, //There for keeping reference to the library
	pub file: String,

	needs_load: bool,
	next_page: String,
	navigation_state: Vec<String>
}
impl Plugin {
	pub fn load(&mut self, kind: NavigationAction, size: u32) -> Result<Vec<YaffePluginItem>, String> {
		match kind {
			NavigationAction::Initialize => {
				info!("Plugin requested initial load");

				self.navigation_state.clear();
				self.needs_load = true;
			},
			NavigationAction::Refresh => {
				info!("Plugin requested refresh");
				self.needs_load = true;
			},
			NavigationAction::Fetch => {
				info!("Plugin requested append");
			},
			NavigationAction::Back => {
				info!("Plugin requested back action");
				self.navigation_state.pop();
				self.needs_load = self.navigation_state.len() > 0;
			}
		}

		if self.needs_load {
			info!("Calling load_items on plugin");
			let result = self.data.load_items(size, &self.navigation_state, &self.next_page);
			match result {
				Ok(results) => {
					self.next_page = results.next_page;
					self.needs_load = !self.next_page.is_empty();
					return Ok(results.results)
				}
				Err(s) => Err(s)
			}
		} else {
			Ok(vec!())
		}
	}

	pub fn select(&mut self, name: &str, path: &str) -> Option<std::io::Result<std::process::Child>> {
		let action = self.data.on_selected(name, path);
		match action {
			SelectedAction::Load(state) => {
				self.navigation_state.push(state);
				None
			},
			SelectedAction::Start(mut p) => Some(p.spawn()),
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
	initialize: fn() -> Box<dyn YaffePlugin>,
}

pub fn load_plugins(state: &mut crate::YaffeState, directory: &str) {
	if !std::path::Path::new(directory).exists() {
		std::fs::create_dir(directory).log_and_panic();
	}
	let path = std::fs::canonicalize(directory).unwrap();

	for entry in std::fs::read_dir(path).log_and_panic() {
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
				
				info!("Found plugin {}", file);

				let container: Option<Container<PluginWrapper>> = unsafe { Container::load(path.clone()) }.display_failure(&message, state);
				if let Some(cont) = container {
					//Create our YaffePlugin object
					let data = cont.initialize();
					
					//Do any initialization work on the object
					let mut plugin = Plugin { 
						_container: cont, 
						file: file.to_string(),
						needs_load: true,
						next_page: String::from(""),
						data,
						navigation_state: vec!()
					};

					//Ensure all settings are present
					let settings = state.settings.plugin(&plugin.file);
					if plugin.data.initialize(&settings).display_failure(&message, state).is_some() {
						state.plugins.push(std::cell::RefCell::new(plugin));
					}
				}
			}
		}
	}
}

pub fn load_plugin_items(kind: NavigationAction, state: &mut crate::YaffeState) {
	if let Some(plugin) = state.get_platform().get_plugin(state) {
		let x = state.settings.get_i32(crate::SettingNames::ItemsPerRow);
		let y = state.settings.get_i32(crate::SettingNames::ItemsPerColumn);

		let items = plugin.borrow_mut().load(kind, (x * y) as u32);
		if let Some(items) = items.display_failure("Error loading plugin", state) {
			let platform = &mut state.platforms[state.selected_platform];
			match kind {
				NavigationAction::Fetch => {}
				_ => platform.apps.clear(),
			}
			for i in items {
				platform.apps.push(crate::Executable::plugin_item(state.selected_platform, i));
			}
		}
	}
}

pub fn unload(plugins: &mut Vec<std::cell::RefCell<Plugin>>) {
	plugins.clear();
}