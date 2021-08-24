use serde_json::Value;
type ServiceResult<T> = Result<T, ServiceError>;

//https://api.thegamesdb.net/

pub struct ServiceResponse<T> {
    pub count: i64,
    pub exact: bool,
    pub results: Vec<T>,
}

impl<T> ServiceResponse<T> {
    fn new(count: i64, exact: bool) -> ServiceResponse<T> {
        ServiceResponse {
            count,
            exact,
            results: vec!(),
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
    ProcessingError,
    InvalidFormat
}

impl From<reqwest::Error> for ServiceError {
    fn from(_: reqwest::Error) -> Self { ServiceError::ProcessingError }
}
impl From<serde_json::Error> for ServiceError {
    fn from(_: serde_json::Error) -> Self { ServiceError::InvalidFormat }
}

fn get_api_key() -> &'static str {
    let data = include_bytes!("../../api_key.txt");
    match std::str::from_utf8(data) {
        Ok(v) => v,
        Err(_) => panic!("Invalid api_key.txt"),
    }
}

fn get_null_string<'a>(value: &'a Value, element: &'a str) -> &'a str {
    if value[element].is_null() { "" } else { value[element].as_str().unwrap() }
}

fn send_request<T: serde::ser::Serialize + ?Sized>(url: &str, parms: &T) -> Result<serde_json::Value, ServiceError> {
    let key = get_api_key();
    let client = reqwest::blocking::Client::new();
    match client.get(url).query(&[("apikey", key)]).query(parms).send() {
        Ok(resp) => {
            if resp.status().is_success() { 
                let text = resp.text()?;
                return Ok(serde_json::from_str(&text)?); 
            }
        }
        Err(e) => { return Err(ServiceError::NetworkError(e)); }
    }

    Err(ServiceError::ProcessingError)
}

pub fn search_game(name: &str, platform: i64) -> ServiceResult<ServiceResponse<GameInfo>> {
    let resp = send_request("https://api.thegamesdb.net/v1.1/Games/ByGameName", 
        &[("name", name), 
          ("fields", "players,overview,rating"), 
          ("filter[platform]", &platform.to_string())])?;

    assert!(resp["data"]["games"].is_array());
    let array = resp["data"]["games"].as_array().unwrap();
    let (count, exact) = get_count_and_exact(array, "game_title", name);
    let mut result = ServiceResponse::new(count, exact);

    if array.len() > 0 {
        let ids = array.iter().map(|v| v["id"].as_i64().unwrap().to_string()).collect::<Vec<String>>();
        let ids = ids.join(",");

        //Get the image data for the games
        let resp = send_request("https://api.thegamesdb.net/v1/Games/Images", 
                    &[("games_id", &ids[..]), ("filter[type]", "banner,boxart")])?;

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
    let resp = send_request("https://api.thegamesdb.net/v1/Platforms/ByPlatformName", &[("name", name)])?;

    assert!(resp["data"]["platforms"].is_array());
    let array = resp["data"]["platforms"].as_array().unwrap();
    let (count, exact) = get_count_and_exact(array, "name", name);
    let mut result = ServiceResponse::new(count, exact);

    for value in array {
        result.results.push(PlatformInfo::new(value));
    }

    Ok(result)
}

fn get_count_and_exact(value: &Vec<serde_json::Value>, element: &str, name: &str) -> (i64, bool) {
    let mut count = 0;
    let mut exact = false;

    for i in value {
        assert!(i[element].is_string());

        count += 1;
        if i[element].as_str().unwrap() == name { exact = true; }
    }
    (count, exact)
}