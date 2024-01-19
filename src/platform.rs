use crate::YaffeState;
use crate::plugins::Plugin;
use crate::logger::PanicLogEntry;
use crate::data::GameInfo;
use super::{Platform, Executable};
use yaffe_lib::YaffePluginItem;
use std::convert::{TryFrom, TryInto};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

#[repr(u8)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PlatformType {
    Emulator,
    Plugin,
    Recents,
}

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Rating {
    Everyone,
    Everyone10,
    Teen,
    Mature,
    AdultOnly,
    NotRated,
    Pending,
}
impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let text = match self {
            Rating::Everyone => "E - Everyone",
            Rating::Everyone10 => "E10+ - Everyone 10+",
            Rating::Teen => "T - Teen",
            Rating::Mature => "M - Mature 17+",
            Rating::AdultOnly => "AO - Adult Only 18+",
            Rating::NotRated => "Not Rated",
            Rating::Pending => "RP - Rating Pending",
        };
        write!(f, "{text}")
    }
}
impl TryFrom<i64> for Rating {
    type Error = ();

    fn try_from(v: i64) -> Result<Self, Self::Error> {
        match v {
            x if x == Rating::Everyone as i64 => Ok(Rating::Everyone),
            x if x == Rating::Everyone10 as i64 => Ok(Rating::Everyone10),
            x if x == Rating::Teen as i64 => Ok(Rating::Teen),
            x if x == Rating::Mature as i64 => Ok(Rating::Mature),
            x if x == Rating::AdultOnly as i64 => Ok(Rating::AdultOnly),
            x if x == Rating::NotRated as i64 => Ok(Rating::NotRated),
            x if x == Rating::Pending as i64 => Ok(Rating::Pending),
            _ => Err(()),
        }
    }
}
impl TryFrom<String> for Rating {
    type Error = ();
    fn try_from(v: String) -> Result<Self, Self::Error> {
        match &v[..] {
            "E - Everyone" => Ok(Rating::Everyone),
            "E10+ - Everyone 10+" => Ok(Rating::Everyone10),
            "T - Teen" => Ok(Rating::Teen),
            "M - Mature 17+" => Ok(Rating::Mature),
            "AO - Adult Only 18+" => Ok(Rating::AdultOnly),
            "Not Rated" => Ok(Rating::NotRated),
            "RP - Rating Pending" => Ok(Rating::Pending),
            _ => Err(()),
        }
    }
}

impl Platform {
    pub fn new(id: i64, name: String) -> Platform {
        super::Platform {
            id: Some(id),
            name,
            apps: vec!(),
            kind: PlatformType::Emulator,
            plugin_index: 0,
        }
    }

    pub fn recents(name: String) -> Platform {
        super::Platform {
            id: None,
            name,
            apps: vec!(),
            kind: PlatformType::Recents,
            plugin_index: 0,
        }
    }

    pub fn plugin(plugin_index: usize, name: String) -> Platform {
        super::Platform {
            id: None,
            name,
            apps: vec!(),
            kind: PlatformType::Plugin,
            plugin_index,
        }
    }

    pub fn get_plugin<'a>(&self, state: &'a YaffeState) -> Option<&'a RefCell<Plugin>> {
        if let PlatformType::Plugin = self.kind {
            let plugin = &state.plugins[self.plugin_index];
            return Some(plugin);
        }
        None
    }

    pub fn get_rom_path(&self) -> std::path::PathBuf {
        std::path::Path::new("./Roms").join(&self.name)
    }
}

impl Executable {
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
            rating: if !item.restricted { Rating::Everyone } else { Rating::Mature },
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
            rating: info.rating.try_into().unwrap(),
        }
    }
}

pub fn get_database_info(state: &mut YaffeState) {
    crate::logger::info!("Refreshing information from database");

    let mut platforms = vec!();
    platforms.push(Platform::recents(String::from("Recent")));
    for p in crate::data::PlatformInfo::get_all() {
        platforms.push(Platform::new(p.id, p.platform));
    }

    for i in 0..platforms.len() {
        refresh_executable(state, &mut platforms, i);
    }

    for (i, p) in state.plugins.iter_mut().enumerate() {
        let name = String::from(p.borrow().name());

        platforms.push(Platform::plugin(i, name));
    }

    state.platforms = platforms;
}

pub fn scan_new_files(state: &mut YaffeState) {
    let mut count = 0;
    let job_id = crate::job_system::generate_job_id();
    for p in &state.platforms {
        if let PlatformType::Emulator = p.kind {
            for entry in std::fs::read_dir(p.get_rom_path()).log_and_panic() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() && is_allowed_file_type(&path) {
                    
                    let file = path.file_name().unwrap().to_string_lossy();
                    let name = path.file_stem().unwrap().to_string_lossy();
                    let name = clean_file_name(&name);
                    crate::logger::info!("Found local game {name}");
                    
                    let lock = state.queue.lock().log_and_panic();
                    let mut queue = lock.borrow_mut();
                    
                    let id = p.id.unwrap();
                    let exists = crate::data::GameInfo::exists(id, &file).log_and_panic();
                    if !exists {
                        crate::logger::info!("{name} not found in database, performing search");
                        let job = crate::Job::SearchGame { id: job_id, exe: file.to_string(), name: name.to_string(), platform: id };
                        queue.send(job).unwrap();

                        count += 1;
                    }
                }
            }
        }
    }

    if count != 0 {
        state.toasts.insert(job_id, format!("Found {count} new files, searching for information..."));
    }
}

fn refresh_executable(state: &mut YaffeState, platforms: &mut [Platform], index: usize) {
    match platforms[index].kind {
        PlatformType::Emulator => {
            let platform = platforms.get_mut(index).unwrap();
            for g in crate::data::GameInfo::get_all(platform.id.unwrap()) {
                let boxart = crate::assets::get_asset_path(&platform.name, &g.name);
                platform.apps.push(Executable::new_game(&g, index, boxart));

            }
            
            platform.apps.sort_by(|a, b| a.name.cmp(&b.name));
        }
        PlatformType::Plugin => {
            //These are not stored from the database, but loaded at runtime
            unreachable!();
        }
        PlatformType::Recents => {
            crate::logger::info!("Getting recent games");

            let max = state.settings.get_i32(crate::SettingNames::ItemsPerRow) as f32 * 
                      state.settings.get_i32(crate::SettingNames::ItemsPerColumn) as f32 *
                      state.settings.get_f32(crate::SettingNames::RecentPageCount);
            platforms[index].apps = crate::data::GameInfo::get_recent(max as i64, platforms);
        }  
    }
}

fn is_allowed_file_type(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().unwrap();
        return !matches!(ext, "ini" | "srm");
    }
    false
}

fn clean_file_name(file: &str) -> &str {
    for (i, c) in file.chars().enumerate() {
        if c == '(' || c == '[' {
            return file[0..i - 1].trim_end();
        }
    }

    file[..].trim_end()
}

pub fn insert_platform(state: &mut YaffeState, data: &crate::data::PlatformInfo) {
    crate::logger::info!("Inserting new platform into database {}", data.platform);

    // Create Roms folder
    let path = Path::new("./Roms");
    if !path.exists() { std::fs::create_dir(path).unwrap(); }
    let path = path.join(&data.platform);
    if !path.exists() { std::fs::create_dir(path).unwrap(); }

    // Create Assets folder
    let path = Path::new("./Assets").join(&data.platform);
    if !path.exists() { std::fs::create_dir(path).unwrap(); }

    crate::data::PlatformInfo::insert(data).log_and_panic();

    state.refresh_list = true;
}

pub fn insert_game(state: &mut YaffeState, info: &crate::data::GameInfo, boxart: PathBuf) {
    crate::logger::info!("Inserting new game into database {}", info.name);

    crate::data::GameInfo::insert(info).log_and_panic();
    
    let plat_name = crate::data::PlatformInfo::get_name(info.platform()).unwrap();

    let file_path = crate::assets::get_asset_path(&plat_name, &info.name);
    let lock = state.queue.lock().log_and_panic();
    let mut queue = lock.borrow_mut();

    queue.send(crate::Job::DownloadUrl { url: boxart, file_path }).unwrap();

    state.refresh_list = true;
}

