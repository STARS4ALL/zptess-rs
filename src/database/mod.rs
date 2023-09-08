pub mod models;
pub mod schema;
pub mod views;

use std::error::Error;

use tracing::{debug, info};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::Sqlite;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/sqlite");

pub type DbConnection = SqliteConnection;
pub type Db = Sqlite;
pub type Pool = diesel::r2d2::Pool<ConnectionManager<DbConnection>>;
pub type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<DbConnection>>;

fn establish_connection(database_url: &str) -> DbConnection {
    DbConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn get_database_version(connection: &mut DbConnection) -> String {
    use self::schema::config_t::dsl::*;

    let sql = config_t
        .filter(section.eq("database"))
        .filter(property.eq("version"))
        //.limit(1)
        .select(value);

    debug!("{:?}", diesel::debug_query::<Db, _>(&sql).to_string());
    let results: Vec<String> = sql.load(connection).expect("Error loading version");
    // We asume in our databases that this configuation entry always exists
    return results[0].clone();
}

fn get_database_uuid(connection: &mut DbConnection) -> String {
    use self::schema::config_t::dsl::*;

    let sql = config_t
        .filter(section.eq("database"))
        .filter(property.eq("uuid"))
        //.limit(1)
        .select(value);

    debug!("{:?}", diesel::debug_query::<Db, _>(&sql).to_string());
    let results: Vec<String> = sql.load(connection).expect("Error loading uuid");

    if results.is_empty() {
        let my_uuid = uuid::Uuid::new_v4().hyphenated().to_string();
        let sql = diesel::insert_into(config_t).values((
            section.eq("database"),
            property.eq("uuid"),
            value.eq(&my_uuid),
        ));
        debug!("{:?}", diesel::debug_query::<Db, _>(&sql).to_string());
        sql.execute(connection).expect("Error saving uuid");
        return my_uuid;
    } else {
        return results[0].clone();
    }
}

fn run_migrations(
    connection: &mut impl MigrationHarness<Db>,
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

// This is for initialization only
// We do not need a connection pool here
pub fn init(database_url: &str) -> DbConnection {
    let mut connection = establish_connection(database_url);
    let _result = run_migrations(&mut connection).expect("Running migrations");
    info!(
        "Opened database {}, version {}, UUID = {}",
        database_url,
        get_database_version(&mut connection),
        get_database_uuid(&mut connection)
    );
    connection
}

// We need a connecttion pool for the tasks
pub fn get_connection_pool(url: &str) -> Pool {
    let manager = ConnectionManager::<DbConnection>::new(url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
