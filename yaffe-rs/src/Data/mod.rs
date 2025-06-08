use crate::logger::PanicLogEntry;
use core::ops::Deref;

mod game;
mod platform;
mod schema;
pub use game::GameInfo;
pub use platform::PlatformInfo;

type QueryResult<T> = Result<T, QueryError>;
#[derive(Debug)]
pub enum QueryError {
    NoResults,
    NoUpdate,
}

pub struct YaffeConnection {
    con: sqlite::Connection,
}
impl YaffeConnection {
    pub fn new() -> YaffeConnection {
        let connection = sqlite::open("./Yaffe.db").log_and_panic();
        YaffeConnection { con: connection }
    }
}
impl Deref for YaffeConnection {
    type Target = sqlite::Connection;

    fn deref(&self) -> &sqlite::Connection { &self.con }
}

#[macro_export]
macro_rules! create_statement {
    ($con:ident, $statement:expr, $($x:expr),*) => {{
        use $crate::logger::PanicLogEntry;
        #[allow(unused_mut)]
        let mut statement = $con.prepare($statement).log_message_and_panic("Unable to prepare statement");
        let mut _i = 1usize;
    $(
        statement.bind((_i, $x)).log_and_panic();
        _i +=1;
    )*
    statement
    }};
}

#[macro_export]
macro_rules! get_column {
    ($row:expr, $ty:ty, $column:expr) => {
        match $row.read::<$ty, _>($column) {
            Ok(v) => v,
            Err(_) => <$ty>::default(),
        }
    };
}

/// Runs the provided function for each row returned from the statement
fn execute_select<F>(mut stmt: sqlite::Statement, mut f: F)
where
    F: FnMut(&sqlite::Statement),
{
    while let Ok(sqlite::State::Row) = stmt.next() {
        f(&stmt)
    }
}

/// Expect one row to be returned from the statement, otherwise errors
fn execute_select_once(stmt: &mut sqlite::Statement) -> QueryResult<()> {
    if let sqlite::State::Row = stmt.next().unwrap() {
        return Ok(());
    }

    Err(QueryError::NoResults)
}

/// Runs an update statement
fn execute_update(mut stmt: sqlite::Statement) -> QueryResult<()> {
    if let sqlite::State::Done = stmt.next().unwrap() {
        return Ok(());
    }

    Err(QueryError::NoUpdate)
}

/// Creates the database if it doesn't exist
pub fn init_database() -> QueryResult<()> {
    if !std::path::Path::new("./Yaffe.db").exists() {
        schema::create_schema("Games", GameInfo::default())?;
        schema::create_schema("Platforms", PlatformInfo::default())?;
    }

    schema::update_schema("Games", GameInfo::default())?;
    schema::update_schema("Platforms", PlatformInfo::default())?;

    Ok(())
}
