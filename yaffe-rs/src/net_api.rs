use serde_json::Value;
use reqwest::blocking::{Response, RequestBuilder, Client};
use std::collections::HashMap;
use std::path::Path;
use crate::data::{PlatformInfo, GameInfo};
type ServiceResult<T> = Result<T, ServiceError>;

pub enum Authentication {
    GamesDb,
    Google,
    None,
}

pub struct GameScrapeResult {
    pub info: GameInfo,
    pub boxart: String,
}
impl crate::ui_control::ListItem for GameScrapeResult {
    fn to_display(&self) -> String {
        self.info.name.clone()
    }
}

//https://api.thegamesdb.net/

pub struct ServiceResponse<T> {
    pub count: usize,
    pub exact_index: isize,
    pub results: Vec<T>,
}

impl<T> ServiceResponse<T> {
    fn new(count: usize, exact_index: isize) -> ServiceResponse<T> {
        ServiceResponse {
            count,
            exact_index,
            results: vec!(),
        }
    }

    pub fn get_exact(&self) -> Option<&T> {
        if self.exact_index != -1 {
            Some(&self.results[self.exact_index as usize])
        } else { 
            None
        }
    }
}

#[derive(Debug)]
pub enum ServiceError {
    NetworkError(reqwest::Error),
    BadStatus(reqwest::StatusCode),
    ProcessingError,
    InvalidFormat,
    Other(&'static str),
}

impl From<reqwest::Error> for ServiceError {
    fn from(_: reqwest::Error) -> Self { ServiceError::ProcessingError }
}
impl From<serde_json::Error> for ServiceError {
    fn from(_: serde_json::Error) -> Self { ServiceError::InvalidFormat }
}

fn get_api_key(t: &Authentication) -> Option<(&str, &str)> {
    let data = match t {
        Authentication::GamesDb => ("apikey", std::str::from_utf8(include_bytes!("../../api_key.txt")).expect("Invalid api_key.txt")),
        Authentication::Google => ("key", std::str::from_utf8(include_bytes!("../../google_api_key.txt")).expect("Invalid google_api_key.txt")),
        Authentication::None => return None,
    };
    Some(data)
}

fn get_null_string<'a>(value: &'a Value, element: &'a str) -> &'a str {
    if value[element].is_null() { "" } else { value[element].as_str().unwrap() }
}

macro_rules! json_request {
    ($t:expr, $url:expr, $parms:expr) => {
        serde_json::from_str::<serde_json::Value>(&crate::net_api::send_request($t, $url, $parms)?.text()?)?
    };
}
macro_rules! data_request {
    ($t:expr, $url:expr, $parms:expr) => {
        crate::net_api::send_request($t, $url, $parms)?.bytes()?
    };
}
/// Sends a request that has no query parameters
pub fn send_request_no_parms(t: Authentication, url: &str) -> Result<Response, ServiceError> {
    let api_key = get_api_key(&t);

    let client = Client::new();
    let mut builder = client.get(url);
    if let Some(key) = api_key {
        builder = builder.query(&[key]);
    }
    send_and_return(builder)
}

/// Sends a request using one or more query parameters
pub fn send_request<T: serde::ser::Serialize + ?Sized>(t: Authentication, url: &str, parms: &T) -> Result<Response, ServiceError> {
    let api_key = get_api_key(&t);

    let client = Client::new();
    let mut builder = client.get(url).query(parms);
    if let Some(key) = api_key {
        builder = builder.query(&[key]);
    }
    send_and_return(builder)
}

fn send_and_return(builder: RequestBuilder) -> Result<Response, ServiceError> {
    match builder.send() {
        Ok(resp) => {
            if resp.status().is_success() { 
                return Ok(resp);
            }
            return Err(ServiceError::BadStatus(resp.status()))
        }
        Err(e) => { return Err(ServiceError::NetworkError(e)); }
    }
}

pub fn search_game(name: &str, exe: String, platform: i64) -> ServiceResult<ServiceResponse<GameScrapeResult>> {
    crate::logger::info!("Searching for game {}", name);

    let resp = json_request!(Authentication::GamesDb, "https://api.thegamesdb.net/v1.1/Games/ByGameName", 
                     &[("name", name), 
                     ("fields", "players,overview,rating"), 
                     ("filter[platform]", &platform.to_string())]);


    assert!(resp["data"]["games"].is_array());
    let array = resp["data"]["games"].as_array().unwrap();

    let (count, exact) = get_count_and_exact(array, "game_title", name);
    let mut result = ServiceResponse::new(count, exact);

    if array.len() > 0 {
        let ids = array.iter().map(|v| v["id"].as_i64().unwrap().to_string()).collect::<Vec<String>>();
        let ids = ids.join(",");

    crate::logger::info!("Getting all images for game {}", name);

    //Get the image data for the games
        let resp = json_request!(Authentication::GamesDb, "https://api.thegamesdb.net/v1/Games/Images", 
                        &[("games_id", &*ids), ("filter[type]", "boxart")]);

        let images = &resp["data"]["images"];
        for game in array {

            let mut boxart = String::from("");
            let id = game["id"].as_i64().unwrap();
            for image in images[id.to_string()].as_array().unwrap() {
                
                let side = get_null_string(image, "side");
                let kind = get_null_string(image, "type");
                let file = get_null_string(image, "filename");
                match (kind, side) {
                    ("boxart", "front") => boxart = String::from(file),
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
    
    let resp = json_request!(Authentication::GamesDb, "https://api.thegamesdb.net/v1/Platforms/ByPlatformName", &[("name", name)]);

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

pub fn check_for_updates(queue: &mut crate::job_system::JobQueue) -> ServiceResult<bool> {
    crate::logger::info!("Checking for updates");

    //For some reason this doesnt work when putting q as a query parameter
    let resp = serde_json::from_str::<serde_json::Value>(&send_request_no_parms(Authentication::Google, "https://www.googleapis.com/drive/v3/files?q='1F7zqYtoUa4AyrBvN02N0QNuabiYCOrhk'+in+parents")?.text()?)?;

    let mut files = HashMap::new();
    assert!(resp["files"].is_array());
    for f in resp["files"].as_array().unwrap() {
        assert!(f["id"].is_string() && f["name"].is_string());

        files.insert(f["name"].as_str().unwrap(), f["id"].as_str().unwrap());
    }

    //Check for remote version file
    let version_file = files.get("version.txt");
    if version_file.is_none() {
        return Err(ServiceError::Other("version.txt not found in remote repository"));
    }
    let url = format!("https://www.googleapis.com/drive/v3/files/{}", *version_file.unwrap());
    let data = data_request!(Authentication::Google, &url, &[("alt", "media")]);

    let version = std::str::from_utf8(&data).unwrap();
    crate::logger::info!("Found remote version {}", version);

    if needs_updating(crate::CARGO_PKG_VERSION, version) {
        crate::logger::info!("Remote version greater than current version. Updating...");

        //Get updated exe file and write to temp location
        let exe_file = files.get("yaffe-rs.exe");
        if exe_file.is_none() {
            return Err(ServiceError::Other("yaffe-rs.exe not found in remote repository"));
        }

        let url = Path::new("https://www.googleapis.com/drive/v3/files/").join(exe_file.unwrap()).join("?alt=media");
        let file = Path::new(crate::UPDATE_FILE_PATH);
        queue.send(crate::job_system::JobType::DownloadUrl((crate::net_api::Authentication::Google, url.to_owned(), file.to_owned()))).unwrap();

        return Ok(true)
    }

    return Ok(false)
}

fn needs_updating(current: &str, updated: &str) -> bool {
    fn parse(version: &str) -> i32 {
        const VERSION_SIZE: usize = 3;

        let mut v = 0;
        for (i, n) in version.split('.').enumerate() {
            let power = VERSION_SIZE - i;
            v += i32::pow(10, power as u32) * str::parse::<i32>(n).unwrap();
        }
        v
    }

    return parse(current) < parse(updated);
}