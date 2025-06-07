use libloading::Library;
use yaffe_lib::{PluginFilter, TileQuery, YaffePlugin};
use crate::logger::{PanicLogEntry, UserMessage, info};
use crate::state::YaffeState;
use std::ops::{DerefMut, Deref};
use std::path::PathBuf;


pub struct Plugin {
	pub filters: Vec<PluginFilter>,
	plugin: Box<dyn YaffePlugin>,

	// Lib must be the last field to ensure it is dropped last
	// This is necessary because YaffePlugin comes from that library
	#[allow(dead_code)]
	lib: Library,
}
impl Deref for Plugin {
    type Target = Box<dyn YaffePlugin>;

    fn deref(&self) -> &Box<dyn YaffePlugin> {
        &self.plugin
    }
}
impl DerefMut for Plugin {
    fn deref_mut(&mut self) -> &mut Box<dyn YaffePlugin> {
        &mut self.plugin
    }
}

fn load(state: &mut YaffeState, path: &mut PathBuf) -> Result<(), Box<dyn std::error::Error>> {
	unsafe {
		let lib = libloading::Library::new(path.clone())?;
		let create_plugin: libloading::Symbol<unsafe extern "C" fn() -> Box<dyn YaffePlugin>> = lib.get(b"create_plugin")?;
		
		let mut plugin = create_plugin();
		info!("Loaded plugin {}", plugin.name());


		path.set_extension("settings");
		let settings = crate::settings::load_settings_from_path(path.clone()).unwrap_or_default();
		let filters = plugin.initialize(&settings).display_failure("Unable to load plugin", state).unwrap_or_default();
		state.plugins.push(Plugin { lib, filters, plugin });
	}
	Ok(())
}

pub fn load_plugins(state: &mut YaffeState, directory: &str) {
	if !std::path::Path::new(directory).exists() {
		std::fs::create_dir(directory).log_and_panic();
	}
	let path = std::fs::canonicalize(directory).unwrap();

	for entry in std::fs::read_dir(path).log_and_panic() {
		let mut path = entry.unwrap().path();

		if let Some(ext) = path.extension() {
			let ext = ext.to_string_lossy();

			if ext == crate::os::lib_ext() && path.is_file() {
				info!("Found plugin {}", path.display());
				load(state, &mut path).display_failure(&format!("Failed to load plugin {path:?}"), state);
			}
		}
	}
}

pub fn load_plugin_items(state: &mut YaffeState, index: usize) {
	let y = state.settings.get_i32(crate::SettingNames::MaxRows);
	let x = state.settings.get_i32(crate::SettingNames::MaxColumns);

	// TODO
	let (filter, value) = if let Some(search) = &state.filter {
		(Some(search.name.clone()), search.get_selected())
	} else {
		(None, None)
	};
	let query = TileQuery {
		filter,
		value,
		limit: (x * (y + 1)) as usize, // Allow for one extra row for scrolling
	};

	if let Some(items) = state.plugins[index].load_tiles(&query).display_failure("Error loading plugin items", state) {
		let group = &mut state.groups[state.selected.group_index];
		for i in items {
			group.tiles.push(crate::Tile::plugin_item(group.id, i));
		}
	}
}

pub fn unload(plugins: &mut Vec<Plugin>) {
	// TODO
	// for p in &mut *plugins {
	// 	std::mem::forget(p.lib);
	// }
	plugins.clear();
}