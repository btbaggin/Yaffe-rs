use core::ops::Deref;
use crate::logger::PanicLogEntry;

mod platform;
mod game;
mod schema;
pub use platform::PlatformInfo;
pub use game::GameInfo;

type QueryResult<T> = Result<T, QueryError>;
#[derive(Debug)]
pub enum QueryError {
    NoResults,
    NoUpdate,
}

pub struct YaffeConnection {
    con: sqlite3::Connection,
}
impl YaffeConnection {
    pub fn new() -> YaffeConnection {
        let connection = sqlite3::open("./Yaffe.db").log_and_panic();
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
        use crate::logger::PanicLogEntry;
        #[allow(unused_mut)]
        let mut statement = $con.prepare($statement).log_message_and_panic("Unable to prepare statement");
        let mut _i = 1usize;
    $(
        statement.bind(_i, $x).log_and_panic();
        _i = _i + 1;
    )*
    statement
    }};
}

/// Runs the provided function for each row returned from the statement
fn execute_select<F>(mut stmt: sqlite3::Statement, mut f: F)
    where F: FnMut(&sqlite3::Statement) {
    while let sqlite3::State::Row = stmt.next().unwrap() {
        f(&stmt)
    }
}

/// Expect one row to be returned from the statement, otherwise errors
fn execute_select_once(stmt: &mut sqlite3::Statement) -> QueryResult<()> {
    if let sqlite3::State::Row = stmt.next().unwrap() {
        return Ok(());
    }

    Err(QueryError::NoResults)
}

/// Runs an update statement
fn execute_update(mut stmt: sqlite3::Statement) -> QueryResult<()> {
    if let sqlite3::State::Done = stmt.next().unwrap() {
        return Ok(());
    }

    Err(QueryError::NoUpdate)
}

/// Creates the database if it doesn't exist
pub fn init_database() -> QueryResult<()> {
    if !std::path::Path::new("./Yaffe.db").exists() {
        let con = YaffeConnection::new();

        const QS_CREATE_GAMES_TABLE: &str = "
        CREATE TABLE \"Games\"
        ( \"ID\" INTEGER, \"Platform\" INTEGER, \"Name\" TEXT, \"Overview\" TEXT, \"Players\" INTEGER, \"Rating\" INTEGER, \"FileName\" TEXT, \"LastRun\" INTEGER )
        ";
        let stmt = create_statement!(con, QS_CREATE_GAMES_TABLE, );
        execute_update(stmt)?;

        const QS_CREATE_PLATFORMS_TABLE: &str = "
        CREATE TABLE \"Platforms\"
        ( \"ID\" INTEGER, \"Platform\" TEXT, \"Path\" TEXT, \"Args\" TEXT, \"Roms\" TEXT )
        ";
        let stmt = create_statement!(con, QS_CREATE_PLATFORMS_TABLE, );
        execute_update(stmt)?;
    }

    schema::update_schema("Games", GameInfo::default())?;
    schema::update_schema("Platforms", PlatformInfo::default())?;

    Ok(())
}






