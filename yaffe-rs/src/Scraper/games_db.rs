use super::{ServiceResult, ServiceResponse, GameScrapeResult, get_null_string};
use crate::{data::{PlatformInfo, GameInfo}, scraper::PlatformScrapeResult};
use std::path::Path;

const GAMESDB_API_KEY: &str = unsafe { std::str::from_utf8_unchecked(include_bytes!("../../api_key.txt")) };

pub fn search_game(id: u64, name: &str, exe: String, platform: i64) -> ServiceResult<ServiceResponse<GameScrapeResult>> {
    crate::logger::info!("Searching for game {name}");

    let resp = crate::json_request!("https://api.thegamesdb.net/v1.1/Games/ByGameName", 
                     &[("name", name), 
                     ("fields", "name,overview,rating,genres"),
                     ("include", "boxart"),
                     ("filter[platform]", &platform.to_string()),
                     ("apikey", GAMESDB_API_KEY)]);


    assert!(resp["data"]["games"].is_array());
    let array = resp["data"]["games"].as_array().unwrap();
    if array.is_empty() { return Ok(ServiceResponse::no_results(id)); }

    let base_url = resp["include"]["boxart"]["base_url"]["medium"].as_str().unwrap();
    let images = resp["include"]["boxart"]["data"].as_object().unwrap();

    let (count, exact) = get_count_and_exact(array, "game_title", name);
    let mut result = ServiceResponse::new(id, String::from(name), count, exact);

    for game in array {
        let id = game["id"].as_i64().unwrap().to_string();

        // Some games don't have images
        let boxart = if let Some(images) = images.get(&id) {
            let game_images = images.as_array().unwrap();

            match game_images.iter().position(|i| i["side"] == "front") {
                // Trim any beginning '/' to ensure the path is interpretted as relative when joining
                Some(i) => String::from(get_null_string(&game_images[i], "filename").trim_start_matches('/')),
                None => String::new(),
            }
        } else {
            String::new()
        };

        let name = String::from(game["game_title"].as_str().unwrap());
        let id = game["id"].as_i64().unwrap();
        let players = game["players"].as_i64().unwrap_or(1);
        let overview = String::from(get_null_string(game, "overview"));
        let rating = String::from(get_null_string(game, "rating"));
        let released = String::from(get_null_string(game, "release_date"));
        let boxart = std::path::Path::new(base_url).join(boxart);

        let info = GameInfo::new(id, name, overview, players, rating, released, exe.clone(), platform);
        result.results.push(GameScrapeResult { info, boxart });
    }

    Ok(result)
}

pub fn search_platform(id: u64, name: &str, path: String, args: String) -> ServiceResult<ServiceResponse<PlatformScrapeResult>> {
    crate::logger::info!("Searching for platform {name}");
    
    let resp = crate::json_request!("https://api.thegamesdb.net/v1/Platforms/ByPlatformName",
                                    &[("name", name),
                                      ("fields", "overview"),
                                      ("apikey", GAMESDB_API_KEY)]);

    assert!(resp["data"]["platforms"].is_array());
    let array = resp["data"]["platforms"].as_array().unwrap();
    if array.is_empty() { return Ok(ServiceResponse::no_results(id)); }

    let (count, exact) = get_count_and_exact(array, "name", name);
    let mut result = ServiceResponse::new(id, String::from(name), count, exact);

    if !array.is_empty() {
        let ids = array.iter().map(|v| v["id"].as_i64().unwrap().to_string()).collect::<Vec<String>>();
        let ids = ids.join(",");

        crate::logger::info!("Getting all images for game {name}");

        //Get the image data for the games
        let resp = crate::json_request!("https://api.thegamesdb.net/v1/Platforms/Images", 
                                        &[("platforms_id", &*ids), ("filter[type]", "boxart"), ("apikey", GAMESDB_API_KEY)]);

        let base_url = resp["data"]["base_url"]["medium"].as_str().unwrap();

        let images = &resp["data"]["images"];
        for platform in array {
            let id = platform["id"].as_i64().unwrap();
            let boxart = images[id.to_string()].as_array().unwrap();
            let boxart = if !boxart.is_empty() {
                // Trim any beginning '/' to ensure the path is interpretted as relative when joining
                String::from(get_null_string(&boxart[0], "filename").trim_start_matches('/'))
            } else {
                String::new()
            };

            let name = String::from(platform["name"].as_str().unwrap());
            let overview = String::from(get_null_string(platform, "overview"));
    
            let info = PlatformInfo::new(id, name, path.clone(), args.clone());
            result.results.push(PlatformScrapeResult { info, overview, boxart: Path::new(base_url).join(boxart) });
        }
    }

    Ok(result)
}

fn get_count_and_exact(value: &Vec<serde_json::Value>, element: &str, name: &str) -> (usize, Option<usize>) {
    let mut count = 0usize;
    let mut exact_index = None;

    for i in value {
        assert!(i[element].is_string());

        if i[element].as_str().unwrap() == name && exact_index.is_none() { 
            exact_index = Some(count);
        }
        count += 1;
    }
    (count, exact_index)
}