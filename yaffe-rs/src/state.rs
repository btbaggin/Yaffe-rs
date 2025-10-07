use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Step;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;

use crate::assets::AssetKey;
use crate::data::GameInfo;
use crate::job_system::ThreadSafeJobQueue;
use crate::logger::{LogEntry, PanicLogEntry, UserMessage};
use crate::modals::Modal;
use crate::modals::RestrictedMode;
use crate::overlay_state::{ExternalProcess, YaffeProcess};
use crate::plugins::Plugin;
use crate::settings::SettingsFile;
use yaffe_lib::{NavigationEntry, PluginFilter, PluginTile, SelectedAction, TileType};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GroupType {
    Emulator,
    Plugin(usize),
    Recents,
}
impl GroupType {
    pub fn allow_edit(&self) -> bool { matches!(self, GroupType::Emulator) }

    pub fn show_count(&self) -> bool { matches!(self, GroupType::Emulator | GroupType::Recents) }
}

#[derive(Debug, Clone)]
pub struct MetadataSearch {
    pub name: String,
    pub options: Vec<String>,
    pub mask: usize,
    pub selected: Option<usize>,
}
impl MetadataSearch {
    pub fn new(name: &str, options: &[&str]) -> MetadataSearch {
        MetadataSearch {
            name: name.to_string(),
            options: options.iter().map(|o| o.to_string()).collect(),
            mask: 0,
            selected: None,
        }
    }

    pub fn from_filter(filter: &PluginFilter) -> MetadataSearch {
        MetadataSearch { name: filter.name.to_string(), options: filter.options.clone(), mask: 0, selected: None }
    }

    pub fn from_range<T: Step + ToString + PartialOrd + Copy>(name: &str, start: T, end: T) -> MetadataSearch {
        MetadataSearch {
            name: name.to_string(),
            options: Self::generate_string_range(start, end),
            mask: 0,
            selected: None,
        }
    }

    pub fn get_selected(&self) -> Option<String> { self.selected.map(|i| self.options[i].clone()) }

    pub fn set_mask(&mut self, tiles: &[Tile]) {
        let mut mask = 0usize;
        for tile in tiles.iter() {
            if let Some(m) = tile.get_metadata(&self.name) {
                let m = m.to_ascii_lowercase();

                for (i, o) in self.options.iter().enumerate() {
                    let o = o.to_ascii_lowercase();
                    if m.starts_with(&o) || m == o {
                        mask |= 1 << i;
                    }
                }
            }
        }
        self.mask = mask;
    }

    pub fn increment_index(&mut self, amount: isize) {
        let mut i = if let Some(i) = self.selected { i as isize } else { -1 };
        //self.index must be assigned in all paths of this loop
        //this loop is guaranteed to end because either the index will hit -1 or self.end
        loop {
            i += amount;
            if i <= -1 {
                self.selected = None;
                return;
            } else if self.mask & 1 << i != 0 {
                self.selected = Some(i as usize);
                return;
            } else if i >= self.options.len() as isize {
                self.selected = None;
                return;
            }
        }
    }

    pub fn item_is_visible(&self, tile: &Tile) -> bool {
        if let Some(i) = self.selected {
            if let Some(m) = tile.get_metadata(&self.name) {
                let m = m.to_ascii_lowercase();

                let o = self.options[i].to_ascii_lowercase();
                if m.starts_with(&o) || m == o {
                    return true;
                }
            }
            false
        } else {
            true
        }
    }

    fn generate_string_range<T: Step + ToString + PartialOrd + Copy>(start: T, end: T) -> Vec<String> {
        let mut v = Vec::new();
        let mut current = start;
        while current <= end {
            v.push(current.to_string());
            if let Some(next) = Step::forward_checked(current, 1) {
                current = next;
            } else {
                break;
            }
        }
        v
    }
}

pub struct TileGroup {
    pub id: i64,
    pub name: String,
    pub tiles: Vec<Tile>,
    pub kind: GroupType,
    pub search: Vec<MetadataSearch>,
}
impl TileGroup {
    pub fn emulator(id: i64, name: String) -> TileGroup {
        super::TileGroup {
            id,
            name,
            tiles: vec![],
            kind: GroupType::Emulator,
            search: vec![
                MetadataSearch::from_range("Players", 1, 4),
                MetadataSearch::new(
                    "Rating",
                    &[
                        "E - Everyone",
                        "E10+ - Everyone 10+",
                        "T - Teen",
                        "M - Mature 17+",
                        "AO - Adult Only 18+",
                        "RP - Rating Pending",
                        "Not Rated",
                        "Restricted",
                    ],
                ),
            ],
        }
    }

    pub fn recents(name: String) -> TileGroup {
        super::TileGroup { id: -1, name, tiles: vec![], kind: GroupType::Recents, search: vec![] }
    }

    pub fn plugin(plugin_index: usize, name: String, filters: &[PluginFilter]) -> TileGroup {
        super::TileGroup {
            id: plugin_index as i64,
            name,
            tiles: vec![],
            kind: GroupType::Plugin(plugin_index),
            search: filters.iter().map(MetadataSearch::from_filter).collect(),
        }
    }

    pub fn get_rom_path(&self) -> PathBuf { std::fs::canonicalize(Path::new("Roms").join(&self.name)).unwrap() }
}

pub struct Tile {
    pub file: String,
    pub tile_type: TileType,
    pub name: String,
    pub description: String,
    pub restricted: bool,
    // We need to store the group on here because recents can be from multiple platforms
    pub group_id: i64,
    pub boxart: AssetKey,
    pub metadata: HashMap<String, String>,
}
impl Tile {
    pub fn plugin_item(group_id: i64, item: PluginTile) -> Self {
        Self {
            file: item.path,
            name: item.name,
            description: item.description,
            tile_type: item.tile_type,
            metadata: HashMap::new(),
            group_id,
            boxart: item.thumbnail.into(),
            restricted: item.restricted,
        }
    }

    pub fn new_game(info: &GameInfo, group_id: i64, boxart: PathBuf) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert(String::from("Players"), info.players.to_string());
        metadata.insert(String::from("Rating"), info.rating.clone());
        metadata.insert(String::from("Released"), info.released.clone());

        let restricted =
            matches!(info.rating.as_str(), "M - Mature 17+" | "Restricted" | "Not Rated" | "AO - Adult Only 18+");
        Self {
            file: info.filename.clone(),
            name: info.name.clone(),
            tile_type: TileType::App,
            description: info.overview.clone(),
            group_id,
            boxart: AssetKey::File(boxart),
            metadata,
            restricted,
        }
    }

    fn get_metadata(&self, key: &str) -> Option<&String> {
        if key == "Name" {
            Some(&self.name)
        } else {
            self.metadata.get(key)
        }
    }

    pub fn get_containing_group_type(&self, state: &YaffeState) -> GroupType {
        let group = state.find_group(self.group_id).unwrap();
        group.kind
    }

    pub fn get_tile_process(
        &self,
        state: &YaffeState,
        group: &TileGroup,
    ) -> Result<Option<YaffeProcess>, Box<dyn std::error::Error>> {
        let child: Box<dyn ExternalProcess> = match group.kind {
            GroupType::Plugin(index) => {
                let plugin = &state.plugins[index];

                match plugin.select_tile(&self.name, &self.file) {
                    SelectedAction::Webview(site) => {
                        let child = crate::utils::yaffe_helper("webview", &[&site]);
                        Box::new(child?) as Box<dyn ExternalProcess>
                    }
                    SelectedAction::Process(mut p) => Box::new(p.spawn()?) as Box<dyn ExternalProcess>,
                }
            }
            GroupType::Emulator | GroupType::Recents => {
                let id = group.id;
                //This should never fail since we got it from the database
                let (path, args) = crate::data::PlatformInfo::get_info(id).log_message_and_panic("Platform not found");
                crate::data::GameInfo::update_last_run(id, &self.file).log("Unable to update game last run");

                let path = std::fs::canonicalize(path)?;
                let mut process = &mut std::process::Command::new(path);
                let exe_path = group.get_rom_path().join(&self.file);

                process = process.arg(exe_path.to_str().unwrap());
                if !args.is_empty() {
                    process = process.args(args.split(' '));
                }
                Box::new(process.spawn()?) as Box<dyn ExternalProcess>
            }
        };
        Ok(Some(YaffeProcess::new(&self.name, self.boxart.clone(), child)))
    }
}

pub struct SelectedItem {
    group_index: usize,
    pub tile_index: usize,
}
impl SelectedItem {
    pub fn new() -> SelectedItem { SelectedItem { group_index: 0, tile_index: 0 } }

    pub fn group_index(&self) -> usize { self.group_index }

    pub fn prev_platform(&mut self) {
        if self.group_index > 0 {
            self.group_index -= 1;
            self.tile_index = 0;
        }
    }

    pub fn next_platform(&mut self, max: usize) {
        if self.group_index < max - 1 {
            self.group_index += 1;
            self.tile_index = 0;
        }
    }
}

pub struct YaffeState {
    process: Rc<RefCell<Option<YaffeProcess>>>,
    pub selected: SelectedItem,
    pub groups: Vec<TileGroup>,
    pub plugins: Vec<Plugin>,
    pub modals: Mutex<Vec<Modal>>,
    pub toasts: HashMap<u64, String>,
    pub queue: ThreadSafeJobQueue,
    pub filter: Option<MetadataSearch>,
    pub restricted_mode: RestrictedMode,
    pub refresh_list: bool,
    pub settings: SettingsFile,
    pub running: bool,
    pub navigation_stack: RefCell<Vec<NavigationEntry>>,
}
impl YaffeState {
    pub fn new(
        process: Rc<RefCell<Option<YaffeProcess>>>,
        settings: SettingsFile,
        queue: ThreadSafeJobQueue,
    ) -> YaffeState {
        YaffeState {
            process,
            selected: SelectedItem::new(),
            groups: vec![],
            plugins: vec![],
            filter: None,
            restricted_mode: RestrictedMode::Off,
            modals: Mutex::new(vec![]),
            toasts: HashMap::new(),
            queue,
            refresh_list: true,
            settings,
            running: true,
            navigation_stack: RefCell::new(Vec::new()),
        }
    }

    pub fn get_selected_group(&self) -> &TileGroup { &self.groups[self.selected.group_index] }

    pub fn get_selected_tile(&self) -> Option<&Tile> {
        let p = &self.get_selected_group();
        if p.tiles.len() > self.selected.tile_index {
            return Some(&p.tiles[self.selected.tile_index]);
        }
        None
    }

    pub fn find_group(&self, id: i64) -> Option<&TileGroup> { self.groups.iter().find(|p| p.id == id) }

    pub fn display_toast(&mut self, id: u64, toast: &str) { self.toasts.insert(id, toast.to_string()); }

    pub fn remove_toast(&mut self, id: &u64) { self.toasts.remove(id); }

    pub fn exit(&mut self) { self.running = false; }

    pub fn is_overlay_active(&self) -> bool { self.process.borrow().is_some() }

    pub fn set_process(&self, process: YaffeProcess) { *self.process.borrow_mut() = Some(process) }

    pub fn navigate_to(&self, tile: &Tile) {
        let mut stack = self.navigation_stack.borrow_mut();
        stack.push(NavigationEntry { path: tile.file.clone(), display: tile.name.clone() });
    }
}

impl crate::ui::WindowState for YaffeState {
    fn on_revert_focus(&mut self) -> bool {
        if self.navigation_stack.borrow_mut().pop().is_some() {
            self.selected.tile_index = 0;

            let group = &mut self.groups[self.selected.group_index];
            group.tiles.clear();
            if let crate::state::GroupType::Plugin(index) = group.kind {
                crate::plugins::load_plugin_items(self, index).display_failure("Error loading plugin items", self);
            }
            return false;
        }
        true
    }
}
