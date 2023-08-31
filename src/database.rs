use std::str;

// Embedding resources in the crate
use rust_embed::RustEmbed;

// SQLite stuff
use rusqlite::{Connection, OpenFlags, Result};

// Logging stuff
use tracing::{info, debug, warn};

use uuid;


const VERSION_QUERY : &str = "SELECT value from config_t WHERE section ='database' AND property = 'version'";
const UUID_QUERY : &str = "SELECT value from config_t WHERE section ='database' AND property = 'uuid'";
const SCHEMA_FILE:  &str = "schema.sql";

// ========================= //

#[derive(RustEmbed)]
#[folder = "resources/sql/"]
#[include = "*.sql"]
#[exclude = "initial/*"]
#[exclude = "updates/*"]
struct SchemaAsset;

#[derive(RustEmbed)]
#[folder = "resources/sql/initial/"]
#[include = "*.sql"]
struct InitialAsset;

#[derive(RustEmbed)]
#[folder = "resources/sql/updates/"]
#[include = "*.sql"]
struct UpdatesAsset;

// =========================== //


fn get_version(conn: &Connection) -> Result<i8> {
    let mut stmt = conn.prepare(VERSION_QUERY)?;
    let mut rows = stmt.query([])?;
    let mut value = String::new();
    while let Some(row) = rows.next()? {
        value = row.get(0)?;
    }
    Ok(value.parse::<i8>().unwrap())
}

fn get_uuid(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare(UUID_QUERY)?;
    let mut rows = stmt.query([])?;
    let mut value = String::new();
    while let Some(row) = rows.next()? {
        value = row.get(0)?;
    }
    Ok(value)
}

// Extracts the NN_xxxx.sql order number
fn file_order(s: &str, n: usize) -> i8 {
    let end = s.chars().map(|c| c.len_utf8()).take(n).sum();
    String::from(&s[..end]).parse::<i8>().unwrap()
}

// Creates a new database file, schema and populate with initial values
fn create(conn: &Connection) -> Result<String> {
    let sql = SchemaAsset::get(SCHEMA_FILE).expect("Schema resource file");
    let sql = str::from_utf8(sql.data.as_ref())?;
    conn.execute_batch(sql).expect("Schema creation failed");
    // Writes an UUID into the config table
    let my_uuid = uuid::Uuid::new_v4().hyphenated().to_string();
    let mut stmt = conn.prepare("INSERT INTO config_t(section,property,value) VALUES(:section,:property,:value)")?;
    stmt.execute(rusqlite::named_params! { ":section": "database", ":property": "uuid", ":value": my_uuid })?;
    // Execute each statement for each file
    for file in InitialAsset::iter() {
        let f = file.as_ref();
        debug!("Initial database population with file {f}");
        let sql = InitialAsset::get(f).expect("Missing initial file");
        let sql = str::from_utf8(sql.data.as_ref())?;
        conn.execute_batch(sql).expect("Initial database population failed");
    }
    Ok(my_uuid)
}


// Updates the database with new SQL files
// Useful to migrate existing databases
fn update(conn: &Connection) -> Result<(i8, String)> {
    let my_uuid = get_uuid(&conn)?;
    let version = get_version(&conn)?;
    for file in UpdatesAsset::iter() {
        let f = file.as_ref();
        let ord = file_order(f, 2);
        if ord > version {
            warn!("Migrating database with SQL file {f}");
            let sql = UpdatesAsset::get(f).expect("Getting SQL file asset");
            let sql = str::from_utf8(sql.data.as_ref())?;
            conn.execute_batch(sql)?;
        } else {
            debug!("Skipping SQL file {f}");
            continue;   
        }
    }
    Ok((version, my_uuid))
}

pub fn init(path: &str) -> Result<Connection> {
    let result = Connection::open_with_flags(
        path,
          OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    );
    let result = match result {
        Ok(conn) => {  // Database already exists, check for schema updates
            let (version, my_uuid) = update(&conn)?; 
            info!("Opened database {path}, version {version:02} with UUID {my_uuid}");
            Ok(conn)
        },
        Err(_) => { // Database does not exists yet, create schema and populate with initial data
            let conn = Connection::open(path)?;
            info!("Creating new database {path}");
            let my_uuid = create(&conn)?;
            info!("Created database {path} with UUID {my_uuid}");
            Ok(conn)
        },
    };
    result
}

