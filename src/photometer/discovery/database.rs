use super::super::super::database::{models::Config, Db, Pool, PooledConnection};
use super::Info;
use diesel::prelude::*;
use tracing::{debug, error, info};

pub struct Discoverer {
    conn: PooledConnection,
}

impl Discoverer {
    pub fn new(pool: &Pool) -> Self {
        Self {
            conn: pool.get().unwrap(),
        }
    }

    pub fn discover(&mut self) -> Info {
        use super::super::super::database::schema::config_t::dsl::*;
        let sql = config_t
            .filter(section.eq("ref-device"))
            .filter(property.ne("endpoint"))
            .filter(property.ne("old_proto"))
            .select(Config::as_select());

        debug!("{:?}", diesel::debug_query::<Db, _>(&sql).to_string());

        let results: Vec<Config> = sql.load(&mut self.conn).expect("Error loading config");
        debug!("{:?}", results);

        let mut info = Info::new();
        for item in results.iter() {
            match item.property.as_str() {
                "model" => info.model = item.value.clone(),
                "name" => info.name = item.value.clone(),
                "mac" => info.mac = item.value.clone(),
                "firmware" => info.firmware = item.value.clone(),
                "sensor" => info.sensor = item.value.clone(),
                "zp" => info.zp = item.value.parse::<f32>().unwrap(),
                "freq_offset" => info.freq_offset = item.value.parse::<f32>().unwrap(),
                &_ => error!("{}", item.property),
            }
        }
        info!("From database: {:#?}", info);
        info
    }
}
