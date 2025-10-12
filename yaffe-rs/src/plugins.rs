use crate::logger::{info, PanicLogEntry};
use crate::{YaffeState, DeferredAction};
use crate::ui::WidgetTree;
use crate::modals::{display_modal_raw, MessageModal, ModalSize};
use libloading::Library;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use yaffe_lib::{LoadItems, NavigationEntry, PluginFilter, TileQuery, YaffePlugin};

pub struct Plugin {
    pub filters: Vec<PluginFilter>,
    plugin: Box<dyn YaffePlugin>,
    navigation_hash: u64,
    done_loading: bool,

    // Lib must be the last field to ensure it is dropped last
    // This is necessary because YaffePlugin comes from that library
    #[allow(dead_code)]
    lib: Library,
}
impl Deref for Plugin {
    type Target = Box<dyn YaffePlugin>;
    fn deref(&self) -> &Box<dyn YaffePlugin> { &self.plugin }
}
impl DerefMut for Plugin {
    fn deref_mut(&mut self) -> &mut Box<dyn YaffePlugin> { &mut self.plugin }
}

fn load(ui: &mut WidgetTree<YaffeState, DeferredAction>, path: &mut PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new(path.clone())?;
        let create_plugin: libloading::Symbol<unsafe extern "C" fn() -> Box<dyn YaffePlugin>> =
            lib.get(b"create_plugin")?;

        let mut plugin = create_plugin();
        let plugin_name = &plugin.name().to_string();
        info!("Loaded plugin {plugin_name}");

        path.set_extension("settings");
        let settings = crate::settings::load_settings_from_path(path.clone(), false).unwrap_or_default();
        plugin
            .initialize(&settings)
            .map_err(|e| {
                let message = MessageModal::from(&format!("Unable to load plugin {plugin_name}: {e:?}"));
                display_modal_raw(ui, "Error", None, message, ModalSize::Half, None);
                e
            })
            .unwrap_or_default();
        let filters = plugin.filters();
        ui.data.plugins.push(Plugin { lib, filters, plugin, navigation_hash: 0, done_loading: false });
    }
    Ok(())
}

pub fn load_plugins(ui: &mut WidgetTree<YaffeState, DeferredAction>, directory: &str) {
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
                let _ = load(ui, &mut path).map_err(|e| {
                    let message = MessageModal::from(&format!("Failed to load plugin {path:?}: {e:?}"));
                    display_modal_raw(ui, "Error", None, message, ModalSize::Half, None);
                    e
                });
            }
        }
    }
}

pub fn load_plugin_items(state: &mut YaffeState, index: usize) -> Result<(), yaffe_lib::PluginError> {
    let y = state.settings.get_i32(crate::SettingNames::MaxRows);
    let x = state.settings.get_i32(crate::SettingNames::MaxColumns);

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

    let plugin = &mut state.plugins[index];
    let stack = state.navigation_stack.borrow();
    if !should_load_plugin(&stack, plugin) {
        return Ok(());
    }

    match plugin.load_tiles(&query, &stack) {
        Err(e) => {
            plugin.done_loading = true;
            return Err(e);
        }
        Ok(load_items) => {
            let group = &mut state.groups[state.selected.group_index()];
            let (items, is_done) = match load_items {
                LoadItems::More(items) => (items, false),
                LoadItems::Done(items) => (items, true),
            };

            for item in items {
                group.tiles.push(crate::Tile::plugin_item(group.id, item));
            }
            plugin.done_loading = is_done;
        }
    }
    Ok(())
}

fn should_load_plugin(stack: &Vec<NavigationEntry>, plugin: &mut Plugin) -> bool {
    let mut s = DefaultHasher::new();
    (*stack).hash(&mut s);
    let hash = s.finish();

    if plugin.navigation_hash != hash {
        plugin.done_loading = false;
        plugin.navigation_hash = hash;
    }
    !plugin.done_loading
}

pub fn unload(plugins: &mut Vec<Plugin>) { plugins.clear(); }
