use core::ops::Deref;
use crate::{Platform, Executable};
use std::convert::TryInto;
use crate::logger::LogEntry;

static QS_GET_PLATFORM: &str = "SELECT Platform, Path, Args, Roms FROM Platforms WHERE ID = @ID";
static QS_GET_PLATFORM_NAME: &str = "SELECT Platform FROM Platforms WHERE ID = @ID";
static QS_GET_ALL_PLATFORMS: &str = "SELECT ID, Platform, Roms FROM Platforms ORDER BY Platform";
static QS_ADD_PLATFORM: &str = "INSERT INTO Platforms ( ID, Platform, Path, Args, Roms ) VALUES ( @PlatformId, @Platform, @Path, @Args, @Roms )";
static QS_UPDATE_PLATFORM: &str = "UPDATE Platforms SET Path = @Path, Args = @Args, Roms = @Roms WHERE ID = @ID";

static QS_GET_GAME: &str = "SELECT ID, Name, Overview, Players, Rating, FileName FROM Games WHERE Platform = @Platform AND FileName = @Game";
static QS_GET_RECENT_GAMES: &str = "SELECT g.Name, g.Overview, g.Players, g.Rating, g.FileName, p.ID, p.Platform FROM Games g, Platforms p WHERE g.Platform = p.ID AND LastRun IS NOT NULL ORDER BY LastRun DESC LIMIT @Max";
static QS_ADD_GAME: &str = "INSERT INTO Games (ID, Platform, Name, Overview, Players, Rating, FileName) VALUES ( @GameId, @Platform, @Name, @Overview, @Players, @Rating, @FileName )";
static QS_UPDATE_GAME_LAST_RUN: &str = "UPDATE Games SET LastRun = strftime('%s', 'now', 'localtime') WHERE Platform = @Platform AND FileName = @Game";

pub static QS_CREATE_GAMES_TABLE: &str = "CREATE TABLE \"Games\" ( \"ID\" INTEGER, \"Platform\" INTEGER, \"Name\" TEXT, \"Overview\" TEXT, \"Players\" INTEGER, \"Rating\" INTEGER, \"FileName\" TEXT, \"LastRun\" INTEGER )";
pub static QS_CREATE_PLATFORMS_TABLE: &str = "CREATE TABLE \"Platforms\" ( \"ID\" INTEGER, \"Platform\" TEXT, \"Path\" TEXT, \"Args\" TEXT, \"Roms\" TEXT )";

type QueryResult<T> = Result<T, QueryError>;
#[derive(Debug)]
pub enum QueryError {
    NoResults,
    NoUpdate,
}

pub struct PlatformData {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub args: String,
    pub folder: String,
}
impl PlatformData {
    pub fn new(info: &crate::server::PlatformInfo, path: String, args: String, folder: String) -> PlatformData {
        PlatformData {
            id: info.id,
            name: info.name.clone(),
            path: path,
            args: args,
            folder: folder,
        }
    }
}
impl crate::modals::ListItem for PlatformData {
    fn to_display(&self) -> String {
        self.name.clone()
    }
}

pub struct GameData {
    pub id: i64,
    pub name: String,
    pub overview: String,
    pub players: i64,
    pub file: String,
    pub rating: i64,
    pub platform: i64,
    pub banner: String,
    pub boxart: String,
}
impl GameData {
    pub fn new(info: &crate::server::GameInfo, file: String, plat_id: i64) -> GameData {
        let rating: crate::platform::Rating = info.rating.clone().try_into().unwrap();
        crate::database::GameData {
            id: info.id,
            name: info.name.clone(),
            overview: info.overview.clone(),
            players: info.players,
            file: file,
            rating: rating as i64,
            platform: plat_id,
            banner: info.banner.clone(),
            boxart: info.boxart.clone(),
        }
    }
}
impl crate::modals::ListItem for GameData {
    fn to_display(&self) -> String {
        self.name.clone()
    }
}

pub struct YaffeConnection {
    con: sqlite3::Connection,
}
impl YaffeConnection {
    pub fn new() -> YaffeConnection {
        let connection = sqlite3::open("./Yaffe.db").log_if_fail();
        YaffeConnection { con:  connection }
    }
}
impl Deref for YaffeConnection {
    type Target = sqlite3::Connection;

    fn deref(&self) -> &sqlite3::Connection {
        &self.con
    }
}

#[macro_export]
macro_rules! create_statement {
    ($con:ident, $statement:expr, $($x:expr),*) => {{
        #[allow(unused_mut)]
        let mut statement = $con.prepare($statement).log_message_if_fail("Unable to prepare statement");
        let mut _i = 1usize;
    $(
        statement.bind(_i, $x).log_if_fail();
        _i = _i + 1;
    )*
    statement
    }};
}

fn execute_select<F>(mut stmt: sqlite3::Statement, mut f: F)
    where F: FnMut(&sqlite3::Statement) {
    while let sqlite3::State::Row = stmt.next().unwrap() {
        f(&stmt)
    }
}

fn execute_select_once(stmt: &mut sqlite3::Statement) -> QueryResult<()> {
    if let sqlite3::State::Row = stmt.next().unwrap() {
        return Ok(());
    }

    Err(QueryError::NoResults)
}

fn execute_update(mut stmt: sqlite3::Statement) -> QueryResult<()> {
    if let sqlite3::State::Done = stmt.next().unwrap() {
        return Ok(());
    }

    Err(QueryError::NoUpdate)
}

pub fn create_database() -> QueryResult<()> {
    if !std::path::Path::new("./Yaffe.db").exists() {
        let con = YaffeConnection::new();

        {
            let stmt = create_statement!(con, QS_CREATE_GAMES_TABLE, );
            execute_update(stmt)?;
        }

        {
            let stmt = create_statement!(con, QS_CREATE_PLATFORMS_TABLE, );
            execute_update(stmt)?;
        }
    }
    Ok(())
}

pub(super) fn get_all_platforms() -> Vec<Platform> {
    crate::log_function!();

    let con = YaffeConnection::new();
    let stmt = create_statement!(con, QS_GET_ALL_PLATFORMS, );

    let mut result = vec!();
    result.push(Platform::recents(String::from("Recent")));

    execute_select(stmt, |r| {
        let id = r.read::<i64>(0).unwrap();
        let name = r.read::<String>(1).unwrap();
        let path = r.read::<String>(2).unwrap();
        result.push(Platform::new(id, name, path));
    });

    result
}

pub fn insert_platform(id: i64, name: &str, path: &str, args: &str, folder: &str) -> QueryResult<()> {
    crate::log_function!(id, name, path, args, folder);
    let con = YaffeConnection::new();
    let stmt = create_statement!(con, QS_ADD_PLATFORM, id, name, path, args, folder);

    execute_update(stmt)?;
    Ok(())
}

pub fn update_platform(platform: i64, path: &str, args: &str, folder: &str) -> QueryResult<()> {
    let con = YaffeConnection::new();

    let stmt = create_statement!(con, QS_UPDATE_PLATFORM, path, args, folder, platform);
    execute_update(stmt)
}

pub fn get_platform_name(platform: i64) -> QueryResult<String> {
    let con = YaffeConnection::new();
    let mut stmt = create_statement!(con, QS_GET_PLATFORM_NAME, platform);
    execute_select_once(&mut stmt)?;
    Ok(stmt.read::<String>(0).unwrap())
}

pub fn get_platform_info(platform: i64) -> QueryResult<(String, String, String)> {
    crate::log_function!(platform);

    let con = YaffeConnection::new();
    let mut stmt = create_statement!(con, QS_GET_PLATFORM, platform);
    execute_select_once(&mut stmt)?;

    Ok((stmt.read::<String>(1).unwrap(), stmt.read::<String>(2).unwrap(), stmt.read::<String>(3).unwrap()))
}

pub(super) fn get_recent_games(max: i64) -> Vec<Executable> {
    crate::log_function!();

    let con = YaffeConnection::new();
    let stmt = create_statement!(con, QS_GET_RECENT_GAMES, max);

    let mut result = vec!();
    execute_select(stmt, |r| {
        let name = r.read::<String>(0).unwrap();
        let overview = r.read::<String>(1).unwrap();
        let players = i64::max(1, r.read::<i64>(2).unwrap());
        let rating = r.read::<i64>(3).unwrap();
        let file = r.read::<String>(4).unwrap();
        let platform_name = r.read::<String>(6).unwrap();

        let (boxart, banner) = crate::assets::get_asset_slot(&platform_name, &name);
        result.push(Executable::new_game(file, 
                                         name, 
                                         overview, 
                                         0, //TODO this needs to be right 
                                         players as u8, 
                                         rating.try_into().log_message_if_fail("Unknown rating value"), 
                                         boxart, 
                                         banner));
    });

    result
}

pub(super) fn get_game_info(id: i64, file: &str) -> QueryResult<(String, String, i64, i64)> {
    crate::logger::log_entry_with_message(crate::logger::LogTypes::Information, "getting all applications", file);

    let con = YaffeConnection::new();
    let mut stmt = create_statement!(con, QS_GET_GAME, id, file);

    if let Ok(_) = execute_select_once(&mut stmt) {
        return Ok((stmt.read::<String>(1).unwrap(), stmt.read::<String>(2).unwrap(), stmt.read::<i64>(3).unwrap(), stmt.read::<i64>(4).unwrap()));
    } else {
        Err(QueryError::NoResults)
    }
}

pub(super) fn insert_game(id: i64, platform: i64, name: &str, overview: &str, players: i64, rating: crate::platform::Rating, file: &str) -> QueryResult<()> {
    crate::log_function!(id, platform, name, overview, players, rating, file);
    let con = YaffeConnection::new();
    let stmt = create_statement!(con, QS_ADD_GAME, id, platform, name, overview, players, rating as i64, file);

    execute_update(stmt)
}

pub fn update_game_last_run(exe: &Executable, id: i64) -> QueryResult<()> {
    crate::log_function!();
    let con = YaffeConnection::new();
    let stmt = create_statement!(con, QS_UPDATE_GAME_LAST_RUN, id, &exe.file[..]);
    
    execute_update(stmt)
}