use serde_json::Value;
use reqwest::blocking::{Response, RequestBuilder, Client};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::data::{GameInfo, PlatformInfo};
use crate::logger::{LogEntry, error};

type ServiceResult<T> = Result<T, ServiceError>;

//https://api.thegamesdb.net/
const GOOGLE_API_KEY: &str = unsafe { std::str::from_utf8_unchecked(include_bytes!("../../google_api_key.txt")) };

mod games_db;
pub use games_db::{search_game, search_platform};


pub struct GameScrapeResult {
    pub info: GameInfo,
    pub boxart: PathBuf,
}
impl crate::ui::ListItem for GameScrapeResult {
    fn to_display(&self) -> String {
        self.info.name.clone()
    }
}

pub struct PlatformScrapeResult {
    pub info: PlatformInfo,
    pub overview: String,
    pub boxart: PathBuf,
}
impl crate::ui::ListItem for PlatformScrapeResult {
    fn to_display(&self) -> String {
        self.info.platform.clone()
    }
}

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


fn get_null_string<'a>(value: &'a Value, element: &'a str) -> &'a str {
    if value[element].is_null() { "" } else { value[element].as_str().unwrap() }
}

#[macro_export]
macro_rules! json_request {
    ($url:expr, $parms:expr) => {
        serde_json::from_str::<serde_json::Value>(&$crate::scraper::send_request($url, Some($parms))?.text()?)?
    };
}

#[macro_export]
macro_rules! data_request {
    ($url:expr, $parms:expr) => {
        $crate::scraper::send_request($url, Some($parms))?.bytes()?
    };
}

/// Sends a request using one or more query parameters
pub fn send_request<T: serde::ser::Serialize + ?Sized>(url: &str, parms: Option<&T>) -> Result<Response, ServiceError> {
    let client = Client::new();
    let mut builder = client.get(url);
    if let Some(parms) = parms {
        builder = builder.query(parms)
    };
    send_and_return(builder)
}

fn send_and_return(builder: RequestBuilder) -> Result<Response, ServiceError> {
    match builder.send() {
        Ok(resp) => {
            if resp.status().is_success() { 
                return Ok(resp);
            }
            Err(ServiceError::BadStatus(resp.status()))
        }
        Err(e) => { Err(ServiceError::NetworkError(e)) }
    }
}



pub fn check_for_updates() -> ServiceResult<bool> {
    crate::logger::info!("Checking for updates");

    //For some reason this doesnt work when putting q as a query parameter
    let url = format!("https://www.googleapis.com/drive/v3/files?q='1F7zqYtoUa4AyrBvN02N0QNuabiYCOrhk'+in+parents&key={GOOGLE_API_KEY}");
    let resp = serde_json::from_str::<serde_json::Value>(&send_request::<()>(&url, None)?.text()?)?;

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
    let data = data_request!(&url, &[("alt", "media"), ("key", GOOGLE_API_KEY)]);

    let version = std::str::from_utf8(&data).unwrap();
    crate::logger::info!("Found remote version {}", version);

    if needs_updating(crate::CARGO_PKG_VERSION, version) {
        crate::logger::info!("Remote version greater than current version. Updating...");

        //Get updated exe file and write to temp location
        let exe_file = files.get("yaffe-rs.exe");
        if exe_file.is_none() {
            return Err(ServiceError::Other("yaffe-rs.exe not found in remote repository"));
        }

        let url = Path::new("https://www.googleapis.com/drive/v3/files/").join(exe_file.unwrap()).join(format!("?alt=media&key={GOOGLE_API_KEY}"));
        let file = Path::new(crate::UPDATE_FILE_PATH);
        download_file(url, file.to_owned());

        return Ok(true)
    }

    Ok(false)
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

    parse(current) < parse(updated)
}

pub fn download_file(url: PathBuf, file_path: PathBuf) {
    match send_request::<()>(url.to_str().unwrap(), None) {
        Ok(bytes) => {
            //Download and write file to disk
            let file = bytes.bytes().unwrap();
            std::fs::write(file_path, file).log("Unable to write downloaded file to disk");
        }
        Err(e) => error!("{:?}", e),
    }
}