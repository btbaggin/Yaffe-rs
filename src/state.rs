use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::assets::AssetKey;
use crate::overlay::OverlayWindow;
use crate::plugins::Plugin;
use crate::restrictions::RestrictedMode;
use crate::ui::{WidgetId, Modal};
use crate::widgets::{PlatformList, SearchInfo};
use crate::job_system::ThreadSafeJobQueue;
use crate::settings::SettingsFile;
use crate::data::GameInfo;
use crate::get_widget_id;
use yaffe_lib::YaffePluginItem;


#[repr(u8)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GroupType {
    Emulator,
    Plugin,
    Recents,
}

pub struct TileGroup {
    pub id: Option<i64>,
    pub name: String,
    pub apps: Vec<Tile>,
    pub kind: GroupType,
    pub plugin_index: usize,
}
impl TileGroup {
    pub fn new(id: i64, name: String) -> TileGroup {
        super::TileGroup {
            id: Some(id),
            name,
            apps: vec!(),
            kind: GroupType::Emulator,
            plugin_index: 0,
        }
    }

    pub fn recents(name: String) -> TileGroup {
        super::TileGroup {
            id: None,
            name,
            apps: vec!(),
            kind: GroupType::Recents,
            plugin_index: 0,
        }
    }

    pub fn plugin(plugin_index: usize, name: String) -> TileGroup {
        super::TileGroup {
            id: None,
            name,
            apps: vec!(),
            kind: GroupType::Plugin,
            plugin_index,
        }
    }

    pub fn get_plugin<'a>(&self, state: &'a YaffeState) -> Option<&'a RefCell<Plugin>> {
        if let GroupType::Plugin = self.kind {
            let plugin = &state.plugins[self.plugin_index];
            return Some(plugin);
        }
        None
    }

    pub fn get_rom_path(&self) -> std::path::PathBuf {
        std::path::Path::new("./Roms").join(&self.name)
    }
}

pub struct Tile {
    pub file: String,
    pub name: String,
    pub description: String,
    pub rating: String,
    pub released: String,
    pub players: u8,
    pub platform_index: usize,
    pub boxart: AssetKey,
}
impl Tile {
    pub fn plugin_item(platform_index: usize, item: YaffePluginItem) -> Self {
        let boxart = match item.thumbnail {
            yaffe_lib::PathType::Url(s) => {
                crate::assets::AssetKey::Url(s)
            },
            yaffe_lib::PathType::File(s) => {
                let canon = std::fs::canonicalize(std::path::Path::new("./plugins").join(s)).unwrap();
                crate::assets::AssetKey::File(canon)
            },
        };

        Self {
            file: item.path,
            name: item.name,
            description: item.description,
            released: String::new(),
            platform_index,
            boxart,
            players: 1,
            rating: if !item.restricted { String::from("Allowed") } else { String::from("Restricted") },
        }
    }

    pub fn new_game(info: &GameInfo, index: usize, boxart: PathBuf) -> Self {
        Self {
            file: info.filename.clone(),
            name: info.name.clone(),
            description: info.overview.clone(),
            platform_index: index,
            boxart: crate::assets::AssetKey::File(boxart),
            released: info.released.clone(),
            players: info.players as u8,
            rating: info.rating.clone(),
        }
    }
}

pub struct YaffeState {
    pub overlay: Rc<RefCell<OverlayWindow>>,
    pub selected_platform: usize,
    pub selected_app: usize,
    pub platforms: Vec<TileGroup>,
    pub plugins: Vec<RefCell<Plugin>>,
    pub focused_widget: WidgetId,
    pub modals: Mutex<Vec<Modal>>,
    pub toasts: HashMap<u64, String>,
    pub queue: ThreadSafeJobQueue,
    pub search_info: SearchInfo,
    pub restricted_mode: RestrictedMode,
    pub refresh_list: bool,
    pub settings: SettingsFile,
    pub running: bool,
}
impl YaffeState {
    pub fn new(overlay: Rc<RefCell<OverlayWindow>>, 
           settings: SettingsFile, 
           queue: ThreadSafeJobQueue) -> YaffeState {
        YaffeState {
            overlay,
            selected_platform: 0,
            selected_app: 0,
            platforms: vec!(),
            plugins: vec!(),
            search_info: SearchInfo::new(),
            focused_widget: get_widget_id!(PlatformList),
            restricted_mode: RestrictedMode::Off,
            modals: Mutex::new(vec!()),
            toasts: HashMap::new(),
            queue,
            refresh_list: true,
            settings,
            running: true,
        }
    }

    pub fn get_platform(&self) -> &TileGroup {
        &self.platforms[self.selected_platform]
    }

    pub fn get_executable(&self) -> Option<&Tile> {
        let p = &self.get_platform();
        if p.apps.len() > self.selected_app { 
            return Some(&p.apps[self.selected_app]);
        }
        None
    }
}