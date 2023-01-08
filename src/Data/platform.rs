use crate::{create_statement, get_column};
use super::{YaffeConnection, execute_update, execute_select, execute_select_once, QueryResult};

crate::table_struct! (
    pub struct PlatformInfo {
        pub id: i64,
        pub platform: String,
        pub path: String,
        pub args: String,
    }
);
impl PlatformInfo {
    pub fn new(id: i64, platform: String, path: String, args: String) -> PlatformInfo {
        PlatformInfo { id, platform, path, args, }
    }

    /// Adds a new platform
    pub fn insert(platform: &PlatformInfo) -> QueryResult<()> {
        const QS_ADD_PLATFORM: &str = "
        INSERT INTO Platforms
        ( ID, Platform, Path, Args, Roms )
        VALUES
        ( @PlatformId, @Platform, @Path, @Args, '' )
        ";
        crate::logger::info!("Inserting new platform into database {}", platform.platform);

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_ADD_PLATFORM, platform.id, &*platform.platform, &*platform.path, &*platform.args);

        execute_update(stmt)
    }

    /// Updates attributes of an existing platform
    pub fn update(platform: i64, exe: &str, args: &str) -> QueryResult<()> {
        const QS_UPDATE_PLATFORM: &str = "UPDATE Platforms SET Path = @Path, Args = @Args WHERE ID = @ID";
        let con = YaffeConnection::new();

        let stmt = create_statement!(con, QS_UPDATE_PLATFORM, exe, args, platform);
        execute_update(stmt)
    }
    
    /// Gets the name of a platform
    pub fn get_name(platform: i64) -> QueryResult<String> {
        const QS_GET_PLATFORM_NAME: &str = "SELECT Platform FROM Platforms WHERE ID = @ID";
        let con = YaffeConnection::new();
        let mut stmt = create_statement!(con, QS_GET_PLATFORM_NAME, platform);
        execute_select_once(&mut stmt)?;
        Ok(get_column!(stmt, String, "Platform"))
    }

    /// Gets Path, and Args of a Platform
    pub fn get_info(platform: i64) -> QueryResult<(String, String)> {
        const QS_GET_PLATFORM: &str = "SELECT Path, Args FROM Platforms WHERE ID = @ID";
        crate::logger::info!("Getting information for platform {}", platform);

        let con = YaffeConnection::new();
        let mut stmt = create_statement!(con, QS_GET_PLATFORM, platform);
        execute_select_once(&mut stmt)?;

        Ok((get_column!(stmt, String, "Path"), get_column!(stmt, String, "Args")))
    }

    /// Gets all saved platforms
    pub fn get_all() -> Vec<PlatformInfo> {
        const QS_GET_ALL_PLATFORMS: &str = "SELECT ID, Platform, Path, Args FROM Platforms ORDER BY Platform";
        crate::logger::info!("Loading all platforms from database");

        let con = YaffeConnection::new();
        let stmt = create_statement!(con, QS_GET_ALL_PLATFORMS, );

        let mut result = vec!();
        execute_select(stmt, |r| {
            let id = get_column!(r, i64, "ID");
            let platform = get_column!(r, String, "Platform");
            let path = get_column!(r, String, "Path");
            let args = get_column!(r, String, "Args");
            result.push(PlatformInfo {
                id,
                platform,
                path,
                args,
            });
        });

        result
    }
}
