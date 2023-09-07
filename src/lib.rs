pub mod argparse;
pub mod database;
pub mod logging;
pub mod photometer;

use dotenvy::dotenv;
use std::env;

const DATABASE_URL: &'static str = "DATABASE_URL";

pub fn get_database_url() -> String {
    dotenv().ok();
    env::var(DATABASE_URL).expect("DATABASE_URL must be set")
}
