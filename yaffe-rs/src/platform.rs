use crate::database::*;
use crate::assets::AssetPathType;
use crate::{YaffeState};
use crate::plugins::Plugin;
use crate::logger::PanicLogEntry;
use super::{Platform, Executable};
use std::convert::{TryFrom, TryInto};
use std::cell::RefCell;
use std::path::Path;

#[repr(u8)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PlatformType {
    Emulator,
    Plugin,
    Recents,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Rating {
    Everyone,
    Everyone10,
    Teen,
    Mature,
    AdultOnly,
    NotRated,
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
            _ => Err(()),
        }
    }
}

impl Platform {
    pub fn new(id: i64, name: String) -> Platform {
        super::Platform {
            id: Some(id),
            name: name,
            apps: vec!(),
            kind: PlatformType::Emulator,
            plugin_index: 0,
        }
    }

    pub fn recents(name: String) -> Platform {
        super::Platform {
            id: None,
            name: name,
            apps: vec!(),
            kind: PlatformType::Recents,
            plugin_index: 0,
        }
    }

    pub fn plugin(index: usize, name: String) -> Platform {
        super::Platform {
            id: None,
            name: name,
            apps: vec!(),
            kind: PlatformType::Plugin,
            plugin_index: index,
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
    pub fn plugin_item(platform_index: usize, item: yaffe_plugin::YaffePluginItem) -> Executable {
        let (boxart, banner) = match item.thumbnail {
            yaffe_plugin::PathType::Url(s) => {
                (AssetPathType::Url(s.clone()), AssetPathType::Url(s))
            },
            yaffe_plugin::PathType::File(s) => {
                let canon = std::fs::canonicalize(format!("./plugins/{}", s)).unwrap();
                let path = canon.to_string_lossy();
                (AssetPathType::File(path.to_string()), AssetPathType::File(path.to_string()))
            },
        };

        super::Executable {
            file: item.path,
            name: item.name,
            description: item.description,
            platform_index: platform_index,
            boxart,
            banner,
            players: 1,
            rating: if !item.restricted { Rating::Everyone } else { Rating::Mature },
        }
    }

    pub fn new_game(file: String, 
                    name: String, 
                    description: String,
                    platform_index: usize, 
                    players: u8,
                    rating: Rating,
                    boxart: String, 
                    banner: String) -> Executable {
        Executable {
            file,
            name,
            description,
            platform_index,
            boxart: AssetPathType::File(boxart),
            banner: AssetPathType::File(banner),
            players,
            rating,
        }
    }
}

pub fn get_database_info(state: &mut YaffeState) {
    crate::logger::info!("Refreshing information from database");

    create_database().log_message_and_panic("Unable to create database");
    let mut platforms = get_all_platforms();

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
    let state_ptr = crate::RawDataPointer::new(state);
    for p in &state.platforms {
        if let PlatformType::Emulator = p.kind {
            for entry in std::fs::read_dir(p.get_rom_path()).log_and_panic() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() && is_allowed_file_type(&path) {
                    
                    let file = path.file_name().unwrap().to_string_lossy();
                    let name = path.file_stem().unwrap().to_string_lossy();
                    let name = clean_file_name(&name);
                    crate::logger::info!("Found local game {}", name);
                    
                    let lock = state.queue.lock().log_and_panic();
                    let mut queue = lock.borrow_mut();
                    
                    let id = p.id.unwrap();
                    let exists = get_game_exists(id, &file).log_and_panic();
                    if !exists {
                        crate::logger::info!("{} not found in database, performing search", name);
                        queue.send(crate::JobType::SearchGame((state_ptr, file.to_string(), name.to_string(), id))).unwrap();
                    }
                }
            }
        }
    }
}

fn refresh_executable(state: &mut YaffeState, platforms: &mut Vec<Platform>, index: usize) {
    match platforms[index].kind {
        PlatformType::Emulator => {
            let platform = platforms.get_mut(index).unwrap();
            for g in get_all_games(platform.id.unwrap()) {
                let name = g.0;
                let (boxart, banner) = crate::assets::get_asset_path(&platform.name, &name);
                platform.apps.push(Executable::new_game(g.4, 
                            name, 
                            g.1, 
                            index, 
                            g.2 as u8, 
                            g.3.try_into().expect("Something went very wrong"), 
                            boxart.to_string_lossy().to_string(), 
                            banner.to_string_lossy().to_string()));

            }
            
            platform.apps.sort_by(|a, b| a.name.cmp(&b.name));
        }
        PlatformType::Plugin => {
            //These are not stored from the database, but loaded at runtime
            assert!(false);
        }
        PlatformType::Recents => {
            crate::logger::info!("Getting recent games");

            let max = state.settings.get_i32(crate::SettingNames::ItemsPerRow) as f32 * 
                      state.settings.get_i32(crate::SettingNames::ItemsPerColumn) as f32 *
                      state.settings.get_f32(crate::SettingNames::RecentPageCount);
            platforms[index].apps = get_recent_games(max as i64, platforms);
        }  
    }
}

fn is_allowed_file_type(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_str().unwrap();
        return match ext { 
            "ini" | "srm" => false,
            _ => true
        };
    }
    false
}

fn clean_file_name(file: &str) -> &str {
    for (i, c) in file.chars().enumerate() {
        if c == '(' || c == '[' {
            return &file[0..i - 1].trim_end();
        }
    }

    &file[..].trim_end()
}

pub fn insert_platform(state: &mut YaffeState, data: &crate::database::PlatformData) {
    crate::logger::info!("Inserting new platform into database {}", data.name);

    crate::database::insert_platform(data.id, &data.name, &data.args, &data.folder).log_and_panic();
    if !Path::new("./Roms").exists() {
        std::fs::create_dir("./Roms").unwrap();
    }
    let folder = Path::new("./Roms").join(&data.name);
    if !folder.exists() {
        std::fs::create_dir(folder).unwrap();
    }

    state.refresh_list = true;
}

pub fn insert_game(state: &mut YaffeState, data: &crate::database::GameData) {
    crate::logger::info!("Inserting new game into database {}", data.name);

    crate::database::insert_game(data.id, 
                                 data.platform, 
                                 &data.name, 
                                 &data.overview, 
                                 data.players, 
                                 data.rating.try_into().unwrap(), 
                                 &data.file).log_and_panic();
    
    let plat_name = crate::database::get_platform_name(data.platform).unwrap();

    let (boxart, banner) = crate::assets::get_asset_path(&plat_name, &data.name);
    let lock = state.queue.lock().log_and_panic();
    let mut queue = lock.borrow_mut();

    let boxart_url = Path::new("https://cdn.thegamesdb.net/images/medium/").join(data.boxart.clone());
    queue.send(crate::JobType::DownloadUrl((crate::net_api::Authentication::None, boxart_url.to_owned(), boxart))).unwrap();

    let banner_url = Path::new("https://cdn.thegamesdb.net/images/medium/").join(data.banner.clone());
    queue.send(crate::JobType::DownloadUrl((crate::net_api::Authentication::None, banner_url.to_owned(), banner))).unwrap();

    state.refresh_list = true;
}

