use std::fs::File;
use std::io::prelude::*;
use std::fs::OpenOptions;
use serde::Deserialize;
use crate::logger::*;

pub enum MessageTypes<'a> {
    PlatformInfo(&'a str),
    GameInfo((i64, &'a str)),
}

type ServiceResult<T> = Result<T, std::io::Error>;

#[derive(Deserialize)]
pub struct ServiceResponse<T> {
    pub count: i32,
    pub exact: bool,
    pub results: Vec<T>,
}

#[derive(Deserialize)]
pub struct GameInfo {
    pub name: String,
    pub id: i64,
    pub players: i64,
    pub overview: String,
    pub rating: String,
    pub banner: String,
    pub boxart: String,
}

#[derive(Deserialize)]
pub struct PlatformInfo {
    pub id: i64,
    pub name: String,
}


const PATH: &str = "//.//pipe/yaffe";
pub fn start_up() {
    if File::open(&PATH).is_err() {
        let mut process = create_process("YaffeService.exe");
        process.spawn().log_if_fail();
    }
}

pub fn create_process(process: &str) -> std::process::Command {
    #[allow(unused_mut)]
    let mut process = std::process::Command::new(process);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        process.creation_flags(CREATE_NO_WINDOW);
    }

    process
}

pub fn send_message(message: MessageTypes) -> ServiceResult<Vec<u8>> {
    let message = match message {
        MessageTypes::PlatformInfo(name) => serde_json::json!({
            "type": 0,
            "name": name
        }),
        MessageTypes::GameInfo((plat, game)) => serde_json::json!({
            "type": 1,
            "platform": plat,
            "name": game
        }),
    };

    log_entry_with_message(LogTypes::Information, "Sending service message", &serde_json::to_string(&message).unwrap());

    let mut f;
    let mut counter = 0;
    loop {
        match OpenOptions::new().write(true).read(true).open(&PATH) {
            Err(err) => {
                //231 means all pipes are busy. 
                //just wait in case we have just sent a ton of requests at once
                if let Some(231) = err.raw_os_error() {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                } else {
                    log_entry(LogTypes::Error, err);
                }
                counter += 1;
                if counter > 10 { return Err(std::io::Error::from(std::io::ErrorKind::Other)); }
            }
            Ok(file) => { f = file; break; },
        }
    };

    f.write_all(&serde_json::to_vec(&message).unwrap()).unwrap();

    let mut buffer = vec!();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn shutdown() {
    let message =  serde_json::json!({
        "type": 2
    });

    if let Ok(mut f) = OpenOptions::new().write(true).read(true).open(&PATH) {
        if let Err(e) = f.write_all(&serde_json::to_vec(&message).unwrap()) {
            log_entry_with_message(LogTypes::Warning, e, "Unable to shut down YaffeService");
        }
    }
}