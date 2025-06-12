use crate::logger::PanicLogEntry;
use crate::state::{GroupType, Tile, TileGroup};
use crate::YaffeState;
use std::path::{Path, PathBuf};

pub fn get_database_info(state: &mut YaffeState) {
    crate::logger::info!("Refreshing information from database");

    let mut platforms = vec![];
    platforms.push(TileGroup::recents(String::from("Recent")));
    for p in crate::data::PlatformInfo::get_all() {
        platforms.push(TileGroup::emulator(p.id, p.platform));
    }

    for p in platforms.iter_mut() {
        refresh_executable(state, p);
    }

    for (i, p) in state.plugins.iter_mut().enumerate() {
        let name = String::from(p.name());

        platforms.push(TileGroup::plugin(i, name, &p.filters));
    }

    state.groups = platforms;
}

pub fn scan_new_files(state: &mut YaffeState) {
    let mut count = 0;
    let job_id = crate::job_system::generate_job_id();
    for p in &state.groups {
        if let GroupType::Emulator = p.kind {
            for entry in std::fs::read_dir(p.get_rom_path()).log_and_panic() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_file() && is_allowed_file_type(&path) {
                    let file = path.file_name().unwrap().to_string_lossy();
                    crate::logger::info!("Found local game {file}");

                    let exists = crate::data::GameInfo::exists(p.id, &file).log_and_panic();
                    if !exists {
                        let name = path.file_stem().unwrap().to_string_lossy();
                        let name = clean_file_name(&name);
                        let name = name.trim();

                        crate::logger::info!("{name} not found in database, performing search");
                        let job = crate::Job::SearchGame {
                            id: job_id,
                            exe: file.to_string(),
                            name: name.to_string(),
                            platform: p.id,
                        };
                        state.start_job(job);

                        count += 1;
                    }
                }
            }
        }
    }

    if count != 0 {
        state.display_toast(job_id, &format!("Found {count} new files, searching for information..."));
    }
}

fn refresh_executable(state: &mut YaffeState, platform: &mut TileGroup) {
    match platform.kind {
        GroupType::Emulator => {
            for g in crate::data::GameInfo::get_all(platform.id) {
                let boxart = crate::assets::get_asset_path(&platform.name, &g.name);
                platform.tiles.push(Tile::new_game(&g, platform.id, boxart));
            }

            platform.tiles.sort_by(|a, b| a.name.cmp(&b.name));
        }
        GroupType::Plugin(_) => {
            //These are not stored from the database, but loaded at runtime
            unreachable!();
        }
        GroupType::Recents => {
            crate::logger::info!("Getting recent games");

            let max = state.settings.get_f32(crate::SettingNames::RecentPageCount);
            platform.tiles = crate::data::GameInfo::get_recent(max as i64);
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

fn clean_file_name(file: &str) -> String {
    let mut i = 0;
    let mut index = 0;
    let mut cleaned_file = String::new();
    let mut chars = file.chars();
    while let Some(c) = chars.next() {
        match c {
            ',' => {
                // When we encounter a comma we want to take the next word and move it to the front
                // eg Legend of Zelda, The

                // Take the string up to this point
                cleaned_file.push_str(&file[index..i]);
                // Move past the comma
                chars.next();
                i += 1;

                // Find the word boundary (or end of string)
                let ii = match chars.position(|cc| cc == ' ') {
                    Some(ii) => ii,
                    None => file.len() - i - 1,
                } + 1;
                // Insert it to the beginning of the string, add a space
                cleaned_file.insert_str(0, &file[i..i + ii]);
                cleaned_file.insert(ii, ' ');
                // Move positions after that word
                i += ii;
                index += i;
            }
            '(' | '[' =>
            /* These have country or language, ignore eg (USA)*/
            {
                break
            }
            _ => {}
        }
        i += 1;
    }
    // If we ended with a comma word, we would have moved past the end of the string
    if index < file.len() {
        cleaned_file.push_str(&file[index..i]);
    }

    cleaned_file
}

pub fn insert_platform(state: &mut YaffeState, data: &crate::data::PlatformInfo) {
    crate::logger::info!("Inserting new platform into database {}", data.platform);

    // Create Roms folder
    let path = Path::new("./Roms");
    if !path.exists() {
        std::fs::create_dir(path).unwrap();
    }
    let path = path.join(&data.platform);
    if !path.exists() {
        std::fs::create_dir(path).unwrap();
    }

    // Create Assets folder
    let path = Path::new("./Assets").join(&data.platform);
    if !path.exists() {
        std::fs::create_dir(path).unwrap();
    }

    crate::data::PlatformInfo::insert(data).log_and_panic();

    state.refresh_list = true;
}

pub fn insert_game(state: &mut YaffeState, info: &crate::data::GameInfo, boxart: PathBuf) {
    crate::logger::info!("Inserting new game into database {}", info.name);

    crate::data::GameInfo::insert(info).log_and_panic();

    let plat_name = crate::data::PlatformInfo::get_name(info.platform()).unwrap();

    let file_path = crate::assets::get_asset_path(&plat_name, &info.name);
    state.start_job(crate::Job::DownloadUrl { url: boxart, file_path });

    state.refresh_list = true;
}
