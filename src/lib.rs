pub mod database;
pub mod logging;

use dotenvy::dotenv;
use std::env;

const DATABASE_URL: &str = "DATABASE_URL";

pub fn get_database_url() -> String {
    dotenv().ok();
    env::var(DATABASE_URL).expect("DATABASE_URL must be set")
}
