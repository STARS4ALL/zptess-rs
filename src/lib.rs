pub mod database;
pub mod logging;
pub mod photometer;
pub mod statistics;

use chrono::prelude::*;
use dotenvy::dotenv;
use std::env;

// let _tstamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
pub type Timestamp = DateTime<Utc>;
pub type Sample = (Timestamp, photometer::payload::Payload);

pub enum Model {
    Tessw,
    Tas,
    Tessp,
}

pub enum Role {
    Refe,
    Test,
}

const DATABASE_URL: &str = "DATABASE_URL";

pub fn get_database_url() -> String {
    dotenv().ok();
    env::var(DATABASE_URL).expect("DATABASE_URL must be set")
}
