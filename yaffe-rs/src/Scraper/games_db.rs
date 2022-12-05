use super::{ServiceResult, ServiceResponse, GameScrapeResult, get_null_string};
use crate::data::{PlatformInfo, GameInfo};

const GAMESDB_API_KEY: &'static str = unsafe { std::str::from_utf8_unchecked(include_bytes!("../../../api_key.txt")) };

pub fn search_game(name: &str, exe: String, platform: i64) -> ServiceResult<ServiceResponse<GameScrapeResult>> {
    crate::logger::info!("Searching for game {}", name);

    let resp = crate::json_request!("https://api.thegamesdb.net/v1.1/Games/ByGameName", 
                     &[("name", name), 
                     ("fields", "players,overview,rating"), 
                     ("filter[platform]", &platform.to_string()),
                     ("apikey", GAMESDB_API_KEY)]);


    assert!(resp["data"]["games"].is_array());
    let array = resp["data"]["games"].as_array().unwrap();

    let (count, exact) = get_count_and_exact(array, "game_title", name);
    let mut result = ServiceResponse::new(count, exact);

    if array.len() > 0 {
        let ids = array.iter().map(|v| v["id"].as_i64().unwrap().to_string()).collect::<Vec<String>>();
        let ids = ids.join(",");

        crate::logger::info!("Getting all images for game {}", name);

        //Get the image data for the games
        let resp = crate::json_request!("https://api.thegamesdb.net/v1/Games/Images", 
                        &[("games_id", &*ids), ("filter[type]", "boxart"), ("apikey", GAMESDB_API_KEY)]);

        let images = &resp["data"]["images"];
        for game in array {

            let mut boxart = String::from("");
            let id = game["id"].as_i64().unwrap();
            for image in images[id.to_string()].as_array().unwrap() {
                
                let side = get_null_string(image, "side");
                let kind = get_null_string(image, "type");
                match (kind, side) {
                    ("boxart", "front") => boxart = String::from(get_null_string(image, "filename")),
                    (_, _) => {},
                }
            }

            let name = String::from(game["game_title"].as_str().unwrap());
            let id = game["id"].as_i64().unwrap();
            let players = game["players"].as_i64().unwrap();
            let overview = String::from(get_null_string(game, "overview"));
            let rating = String::from(get_null_string(game, "rating"));

            let info = GameInfo::new(id, name, overview, players, rating, exe.clone(), platform);
            result.results.push(GameScrapeResult { info, boxart });
        }
    }

    Ok(result)
}

pub fn search_platform(name: &str, path: String, args: String) -> ServiceResult<ServiceResponse<PlatformInfo>> {
    crate::logger::info!("Searching for platform {}", name);
    
    let resp = crate::json_request!("https://api.thegamesdb.net/v1/Platforms/ByPlatformName", &[("name", name), ("apikey", GAMESDB_API_KEY)]);

    assert!(resp["data"]["platforms"].is_array());
    let array = resp["data"]["platforms"].as_array().unwrap();
    let (count, exact) = get_count_and_exact(array, "name", name);
    let mut result = ServiceResponse::new(count, exact);

    for value in array {
        let id = value["id"].as_i64().unwrap();
        let name = String::from(value["name"].as_str().unwrap());
        result.results.push(PlatformInfo::new(id, name, path.clone(), args.clone()));
    }

    Ok(result)
}

fn get_count_and_exact(value: &Vec<serde_json::Value>, element: &str, name: &str) -> (usize, isize) {
    let mut count: usize = 0;
    let mut exact_index: isize = -1;

    for i in value {
        assert!(i[element].is_string());

        if i[element].as_str().unwrap() == name { 
            exact_index = count as isize;
        }
        count += 1;
    }
    (count, exact_index)
}