use serde_json::Value;
use std::collections::HashMap;
type ServiceResult<T> = Result<T, ServiceError>;

enum RequestType {
    GamesDb,
    Google,
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

pub struct GameInfo {
    pub name: String,
    pub id: i64,
    pub players: i64,
    pub overview: String,
    pub rating: String,
    pub banner: String,
    pub boxart: String,
}

impl GameInfo {
    fn new(banner: String, boxart: String, value: &Value) -> GameInfo {
        GameInfo {
            name: String::from(value["game_title"].as_str().unwrap()),
            id: value["id"].as_i64().unwrap(),
            players: value["players"].as_i64().unwrap(),
            overview: String::from(get_null_string(value, "overview")),
            rating: String::from(get_null_string(value, "rating")),
            banner,
            boxart,
        }
    }
}

pub struct PlatformInfo {
    pub id: i64,
    pub name: String,
}

impl PlatformInfo {
    fn new(value: &Value) -> PlatformInfo {
        PlatformInfo {
            id: value["id"].as_i64().unwrap(),
            name: String::from(value["name"].as_str().unwrap()),
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

fn get_api_key(t: &RequestType) -> &str {
    let data = match t {
        RequestType::GamesDb => std::str::from_utf8(include_bytes!("../../api_key.txt")),
        RequestType::Google => std::str::from_utf8(include_bytes!("../../google_api_key.txt")),
    } ;
    match data {
        Ok(v) => v,
        Err(_) => panic!("Invalid api_key.txt"),
    }
}

fn get_null_string<'a>(value: &'a Value, element: &'a str) -> &'a str {
    if value[element].is_null() { "" } else { value[element].as_str().unwrap() }
}

macro_rules! json_request {
    ($t:expr, $url:expr, $parms:expr) => {
        serde_json::from_str::<serde_json::Value>(&send_request($t, $url, $parms)?.text()?)?
    };
}
macro_rules! data_request {
    ($t:expr, $url:expr, $parms:expr) => {
        send_request($t, $url, $parms)?.bytes()?
    };
}

fn send_request<T: serde::ser::Serialize + ?Sized>(t: RequestType, url: &str, parms: &T) -> Result<reqwest::blocking::Response, ServiceError> {
    let api_key = get_api_key(&t);
    let key = match t {
        RequestType::GamesDb => [("apikey", api_key)],
        RequestType::Google => [("key", api_key)],
    };

    let client = reqwest::blocking::Client::new();
    match client.get(url).query(parms).query(&key).send() {
        Ok(resp) => {
            if resp.status().is_success() { 
                return Ok(resp);
            }
            return Err(ServiceError::BadStatus(resp.status()))
        }
        Err(e) => { return Err(ServiceError::NetworkError(e)); }
    }
}

pub fn search_game(name: &str, platform: i64) -> ServiceResult<ServiceResponse<GameInfo>> {
    crate::logger::log_entry(crate::logger::LogTypes::Fine, format!("Searching for game {}", name));

    let resp = json_request!(RequestType::GamesDb, "https://api.thegamesdb.net/v1.1/Games/ByGameName", 
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

    crate::logger::log_entry(crate::logger::LogTypes::Fine, format!("Getting all images for game {}", name));

    //Get the image data for the games
        let resp = json_request!(RequestType::GamesDb, "https://api.thegamesdb.net/v1/Games/Images", 
                        &[("games_id", &ids[..]), ("filter[type]", "banner,boxart")]);

        let images = &resp["data"]["images"];
        for game in array {

            let mut banner = String::from("");
            let mut boxart = String::from("");
            let id = game["id"].as_i64().unwrap();
            for image in images[id.to_string()].as_array().unwrap() {
                
                let side = get_null_string(image, "side");
                let kind = get_null_string(image, "type");
                let file = get_null_string(image, "filename");
                match (kind, side) {
                    ("banner", _) => banner = String::from(file),
                    ("boxart", "front") => boxart = String::from(file),
                    (_, _) => {},
                }
            }

            result.results.push(GameInfo::new(banner, boxart, game));
        }
    }

    Ok(result)
}

pub fn search_platform(name: &str) -> ServiceResult<ServiceResponse<PlatformInfo>> {
    crate::logger::log_entry(crate::logger::LogTypes::Fine, format!("Searching for platform {}", name));
    
    let resp = json_request!(RequestType::GamesDb, "https://api.thegamesdb.net/v1/Platforms/ByPlatformName", &[("name", name)]);

    assert!(resp["data"]["platforms"].is_array());
    let array = resp["data"]["platforms"].as_array().unwrap();
    let (count, exact) = get_count_and_exact(array, "name", name);
    let mut result = ServiceResponse::new(count, exact);

    for value in array {
        result.results.push(PlatformInfo::new(value));
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

pub fn check_for_updates() -> ServiceResult<bool> {
    crate::logger::log_entry(crate::logger::LogTypes::Fine, "Checking for updates");

    //For some reason this doesnt work when putting q as a query parameter
    let resp = json_request!(RequestType::Google, "https://www.googleapis.com/drive/v3/files?q='1F7zqYtoUa4AyrBvN02N0QNuabiYCOrhk'+in+parents", &[("", "")]);

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
    let data = data_request!(RequestType::Google, &url, &[("alt", "media")]);

    let version = std::str::from_utf8(&data).unwrap();
    crate::logger::log_entry(crate::logger::LogTypes::Fine, format!("Found remote version {}", version));

    if version != crate::CARGO_PKG_VERSION {
        //Get updated exe file and write to temp location
        let exe_file = files.get("yaffe-rs.exe");
        if exe_file.is_none() {
            return Err(ServiceError::Other("yaffe-rs.exe not found in remote repository"));
        }

        let url = format!("https://www.googleapis.com/drive/v3/files/{}", *exe_file.unwrap());
        let data = data_request!(RequestType::Google, &url, &[("alt", "media")]);

        if let Err(_) = std::fs::write(crate::UPDATE_FILE_PATH, data) {
            return Err(ServiceError::Other("Unable to write updated file"));
        }

        return Ok(true)
    }

    return Ok(false)
}