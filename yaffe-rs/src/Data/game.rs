use std::convert::TryInto;
use crate::create_statement;
use crate::logger::PanicLogEntry;
use super::{YaffeConnection, QueryResult, QueryError, execute_select, execute_select_once, execute_update};
use crate::{Executable, Platform};

crate::table_struct! (
    pub struct GameInfo {
        id: i64,
        pub name: String,
        pub overview: String,
        pub players: i64,
        pub rating: i64,
        filename: String,
        platform: i64,
        lastrun: i64,
    }
);
impl GameInfo {
    pub fn new(id: i64, name: String, overview: String, players: i64, rating: String, filename: String, platform: i64) -> GameInfo {
        let rating: crate::platform::Rating = rating.clone().try_into().unwrap();
        GameInfo { id, name, overview, players, filename, rating: rating as i64, platform, lastrun: 0 }
    }

    pub fn platform(&self) -> i64 { self.platform }

    pub fn get_all(platform: i64) -> Vec<(String, String, i64, i64, String)> {
        const QS_GET_ALL_GAMES: &str = "SELECT ID, Name, Overview, Players, Rating, FileName FROM Games WHERE Platform = @Platform";

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_GET_ALL_GAMES, platform);
    
        let mut result = vec!();
        execute_select(stmt, |r| {
            let name = r.read::<String>(1).unwrap();
            let overview = r.read::<String>(2).unwrap();
            let players = r.read::<i64>(3).unwrap();
            let rating = r.read::<i64>(4).unwrap();
            let filename = r.read::<String>(5).unwrap();
    
            result.push((name, overview, players, rating, filename))
        });
    
        result
    }

    /// Gets Name, Overview, Players, and Rating of a game
    pub fn exists(id: i64, file: &str) -> QueryResult<bool> {
        const QS_GET_GAME_EXISTS: &str = "SELECT COUNT(1) FROM Games WHERE Platform = @Platform AND FileName = @Game";
        crate::logger::info!("Getting all applications for {}", file);

        let con = YaffeConnection::new();
        let mut stmt = create_statement!(con, QS_GET_GAME_EXISTS, id, file);

        if let Ok(_) = execute_select_once(&mut stmt) {
            let count = stmt.read::<i64>(0).unwrap();
            return Ok(count > 0);
        } else {
            Err(QueryError::NoResults)
        }
    }

    /// Gets the most recent games launched from Yaffe
    pub fn get_recent(max: i64, map: &Vec<Platform>) -> Vec<Executable> {
        //TODO could we use lastrun field?
        const QS_GET_RECENT_GAMES: &str = "SELECT g.Name, g.Overview, g.Players, g.Rating, g.FileName, p.ID, p.Platform FROM Games g, Platforms p WHERE g.Platform = p.ID AND LastRun IS NOT NULL ORDER BY LastRun DESC LIMIT @Max";
        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_GET_RECENT_GAMES, max);

        let mut result = vec!();
        execute_select(stmt, |r| {
            let name = r.read::<String>(0).unwrap();
            let overview = r.read::<String>(1).unwrap();
            let players = i64::max(1, r.read::<i64>(2).unwrap());
            let rating = r.read::<i64>(3).unwrap();
            let file = r.read::<String>(4).unwrap();
            let platform_id = r.read::<i64>(5).unwrap();
            let platform_name = r.read::<String>(6).unwrap();

            let boxart = crate::assets::get_asset_path(&platform_name, &name);
            let index = map.iter().position(|s| s.id == Some(platform_id));
            if let Some(index) = index {
                result.push(Executable::new_game(file, 
                    name, 
                    overview, 
                    index,
                    players as u8, 
                    rating.try_into().log_message_and_panic("Unknown rating value"), 
                    boxart.to_string_lossy().to_string()));
            }
        });

        result
    }

    /// Adds a new game
    pub fn insert(game: &GameInfo) -> QueryResult<()> {
        const QS_ADD_GAME: &str = "
        INSERT INTO Games
        (ID, Platform, Name, Overview, Players, Rating, FileName)
        VALUES
        ( @GameId, @Platform, @Name, @Overview, @Players, @Rating, @FileName )
        ";
        crate::logger::info!("Inserting new game into database {}", game.name);

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_ADD_GAME, game.id, game.platform, &*game.name, &*game.overview, game.players, game.rating as i64, &*game.filename);

        execute_update(stmt)
    }

    /// Updates the last run value for a game
    pub fn update_last_run(id: i64, file: &str) -> QueryResult<()> {
        const QS_UPDATE_GAME_LAST_RUN: &str = "
        UPDATE Games
        SET LastRun = strftime('%s', 'now', 'localtime')
        WHERE Platform = @Platform AND FileName = @Game
        ";
        crate::logger::info!("Updating last run for game {}", id);

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_UPDATE_GAME_LAST_RUN, id, file);
        
        execute_update(stmt)
    }
}