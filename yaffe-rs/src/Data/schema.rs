use super::{execute_select, execute_update, YaffeConnection, QueryResult};
use std::collections::HashMap;

pub struct ColumnInfo {
    name: String,
    data_type: String,
}
impl ColumnInfo {
    pub fn new(name: String, data_type: String) -> ColumnInfo {
        ColumnInfo { name: name.to_lowercase(), data_type: data_type.to_lowercase() }
    }
}

pub trait Schema {
    fn get_columns(&self) -> Vec<ColumnInfo>;
}

#[macro_export]
macro_rules! table_struct {
    ($sv:vis struct $name:ident {
        $($v:vis $field_name:ident: $field_type:ty,)+
    }) => {
        #[allow(dead_code)]
        #[derive(Default)]
        $sv struct $name { $($v $field_name: $field_type),+ }
        impl $crate::data::schema::Schema for $name {
            fn get_columns(&self) -> Vec<$crate::data::schema::ColumnInfo> {
                let mut results = vec!();
            $(
                let name = stringify!($field_name).to_owned();
                let data_type = stringify!($field_type).to_owned();
                results.push($crate::data::schema::ColumnInfo::new(name, data_type));
            )*
                results
            }
        }
    };
}

lazy_static::lazy_static! {
    static ref TYPE_MAPPING: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("string", "TEXT");
        m.insert("i64", "INTEGER");
        m
    };
}

pub fn create_schema(table: &str, data: impl Schema) -> QueryResult<()> {
    let columns = data.get_columns();
    let con = YaffeConnection::new();

    let mut create = String::from("CREATE TABLE \"");
    create.push_str(table);
    create.push_str("\" (");
    
    for c in &columns {
        let t = TYPE_MAPPING.get(&c.data_type[..]).unwrap();
        create.push('"');
        create.push_str(&c.name);
        create.push_str("\" ");
        create.push_str(t);
        create.push(',');
    }

    create.pop();
    create.push(')');
    let stmt = crate::create_statement!(con, create, );
    execute_update(stmt)?;
    Ok(())
}

pub fn update_schema(table: &str, data: impl Schema) -> QueryResult<()> {
    let columns = data.get_columns();
    let table_columns = get_table_columns(table);

    let con = YaffeConnection::new();
    for c in &columns {
        let t = TYPE_MAPPING.get(&c.data_type[..]).unwrap();
        if !table_columns.iter().any(|t| t.name == c.name) {
           let stmt = crate::create_statement!(con, format!("ALTER TABLE {} ADD COLUMN {} {}", table, c.name, t), );
           execute_update(stmt)?;
        }
        // SQLite doesn't have a method to modify a column type and we shouldn't really need that
    }

    for t in &table_columns {
        if !columns.iter().any(|c| c.name == t.name) {
           let stmt = crate::create_statement!(con, format!("ALTER TABLE {} DROP COLUMN {}", table, t.name), );
           execute_update(stmt)?;
        }
    }

    Ok(())
}

fn get_table_columns(table: &str) -> Vec<ColumnInfo> {
    let con = YaffeConnection::new();
    let stmt = crate::create_statement!(con, format!("PRAGMA table_info({table});"), );

    let mut table = vec!();
    execute_select(stmt, |r| {
        let name = r.read::<String, _>(1).unwrap();
        let data_type = r.read::<String, _>(2).unwrap();
        table.push(ColumnInfo::new(name, data_type));
    });
    table
}