use crate::create_statement;
use super::{YaffeConnection, QueryResult, QueryError, execute_select, execute_select_once, execute_update};
use crate::{Executable, Platform, get_column};

crate::table_struct! (
    pub struct GameInfo {
        id: i64,
        pub name: String,
        pub overview: String,
        pub players: i64,
        pub rating: String,
        pub released: String,
        pub filename: String,
        pub platform: i64,
        pub lastrun: i64,
    }
);
impl GameInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(id: i64, name: String, overview: String, players: i64,
               rating: String, released: String,
               filename: String, platform: i64) -> GameInfo {
        GameInfo { id, name, overview, players, filename, rating, released, platform, lastrun: 0 }
    }

    fn from_row(row: &sqlite::Statement, platform: i64) -> GameInfo {
        let id = get_column!(row, i64, "id");
        let name = get_column!(row, String, "name");
        let overview = get_column!(row, String, "overview");
        let players = get_column!(row, i64, "players");
        let rating = get_column!(row, String, "rating");
        let released = get_column!(row, String, "released");
        let filename = get_column!(row, String, "filename");
        let lastrun = get_column!(row, i64, "lastrun");

        GameInfo { id, name, overview, players, filename, rating, released, platform, lastrun }
    }

    pub fn platform(&self) -> i64 { self.platform }

    pub fn get_all(platform: i64) -> Vec<GameInfo> {
        const QS_GET_ALL_GAMES: &str = "SELECT id, name, overview, players, rating, released, filename, lastrun FROM Games WHERE platform = @Platform";

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_GET_ALL_GAMES, platform);
    
        let mut result = vec!();
        execute_select(stmt, |r| {
            result.push(GameInfo::from_row(r, platform))
        });
    
        result
    }

    /// Gets Name, Overview, Players, and Rating of a game
    pub fn exists(id: i64, file: &str) -> QueryResult<bool> {
        const QS_GET_GAME_EXISTS: &str = "SELECT COUNT(1) FROM Games WHERE platform = @Platform AND filename = @Game";
        crate::logger::info!("Getting all applications for {}", file);

        let con = YaffeConnection::new();
        let mut stmt = create_statement!(con, QS_GET_GAME_EXISTS, id, file);

        if execute_select_once(&mut stmt).is_ok() {
            let count = get_column!(stmt, i64, 0);
            Ok(count > 0)
        } else {
            Err(QueryError::NoResults)
        }
    }

    /// Gets the most recent games launched from Yaffe
    pub fn get_recent(max: i64, map: &[Platform]) -> Vec<Executable> {
        const QS_GET_RECENT_GAMES: &str = "SELECT g.id, g.name, g.overview, g.players, g.rating, g.filename, g.released, p.id as platformid, p.platform FROM Games g, Platforms p WHERE g.platform = p.id AND lastrun IS NOT NULL ORDER BY lastrun DESC LIMIT @Max";
        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_GET_RECENT_GAMES, max);

        let mut result = vec!();
        execute_select(stmt, |r| {
            let name = get_column!(r, String, "name");
            let platform_name = get_column!(r, String, "platform");
            let platform_id = get_column!(r, i64, "platformid");

            let info = GameInfo::from_row(r, platform_id);
            let boxart = crate::assets::get_asset_path(&platform_name, &name);
            let index = map.iter().position(|s| s.id == Some(platform_id));
            if let Some(index) = index {
                result.push(Executable::new_game(&info, index, boxart));
        }
        });

        result
    }

    /// Adds a new game
    pub fn insert(game: &GameInfo) -> QueryResult<()> {
        const QS_ADD_GAME: &str = "
        INSERT INTO Games
        (id, platform, name, overview, players, rating, filename)
        VALUES
        (@GameId, @Platform, @Name, @Overview, @Players, @Rating, @FileName)
        ";
        crate::logger::info!("Inserting new game into database {}", game.name);

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_ADD_GAME, game.id, game.platform, &*game.name, &*game.overview, game.players, &*game.rating, &*game.filename);

        execute_update(stmt)
    }

    /// Updates the last run value for a game
    pub fn update_last_run(id: i64, file: &str) -> QueryResult<()> {
        const QS_UPDATE_GAME_LAST_RUN: &str = "
        UPDATE Games
        SET lastrun = strftime('%s', 'now', 'localtime')
        WHERE platform = @Platform AND filename = @Game
        ";
        crate::logger::info!("Updating last run for game {}", id);

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_UPDATE_GAME_LAST_RUN, id, file);
        
        execute_update(stmt)
    }
}