pub mod models;
pub mod schema;
pub mod views;

use dotenvy::dotenv;
use std::env;
use std::error::Error;

use tracing::{debug, info};

use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/sqlite");

pub fn establish_connection() -> (SqliteConnection, String) {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    (conn, database_url)
}

pub fn get_database_version(connection: &mut SqliteConnection) -> String {
    use self::schema::config_t::dsl::*;

    let sql = config_t
        .filter(section.eq("database"))
        .filter(property.eq("version"))
        //.limit(1)
        .select(value);

    debug!("{:?}", diesel::debug_query::<Sqlite, _>(&sql).to_string());
    let results: Vec<String> = sql.load(connection).expect("Error loading version");
    // We asume in our databases that this configuation entry always exists
    return results[0].clone();
}

pub fn get_database_uuid(connection: &mut SqliteConnection) -> String {
    use self::schema::config_t::dsl::*;

    let sql = config_t
        .filter(section.eq("database"))
        .filter(property.eq("uuid"))
        //.limit(1)
        .select(value);

    debug!("{:?}", diesel::debug_query::<Sqlite, _>(&sql).to_string());
    let results: Vec<String> = sql.load(connection).expect("Error loading uuid");

    if results.is_empty() {
        let my_uuid = uuid::Uuid::new_v4().hyphenated().to_string();
        let sql = diesel::insert_into(config_t).values((
            section.eq("database"),
            property.eq("uuid"),
            value.eq(&my_uuid),
        ));
        debug!("{:?}", diesel::debug_query::<Sqlite, _>(&sql).to_string());
        sql.execute(connection).expect("Error saving uuid");
        return my_uuid;
    } else {
        return results[0].clone();
    }
}

pub fn run_migrations(
    connection: &mut impl MigrationHarness<Sqlite>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    if connection.has_pending_migration(MIGRATIONS)? {
        info!("Applying pending migrations");
        connection.run_pending_migrations(MIGRATIONS)?;
    }
    Ok(())
}

pub fn init() -> SqliteConnection {
    let (mut connection, url) = establish_connection();
    let _result = run_migrations(&mut connection).expect("Running migrations");
    //show_config(&mut connection);
    //show_batch(&mut connection);
    //let uuid = get_uuid2(&mut connection);
    //println!("{:?}", uuid);
    let uuid = get_database_uuid(&mut connection);
    let version = get_database_version(&mut connection);
    info!(
        "Opened database {}, version {}, UUID = {}",
        url, version, uuid
    );
    connection
}
