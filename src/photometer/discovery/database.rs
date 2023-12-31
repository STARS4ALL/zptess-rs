use super::Info;
use crate::database::{models::Config, Db, Pool};
use anyhow::Result;
use diesel::prelude::*;
use tokio::task;
use tracing::{debug, error};

pub struct Discoverer<'a> {
    pool: &'a Pool,
}

impl<'a> Discoverer<'a> {
    pub fn new(pool: &'a Pool) -> Self {
        Self { pool }
    }

    pub async fn discover(&self) -> Result<Info> {
        use crate::database::schema::config_t::dsl::*;
        let sql = config_t
            .filter(section.eq("ref-device"))
            .filter(property.ne("endpoint"))
            .filter(property.ne("old_proto"))
            .select(Config::as_select());

        debug!("{:?}", diesel::debug_query::<Db, _>(&sql).to_string());
        let mut conn1 = self.pool.get()?;
        let results =
            task::spawn_blocking(move || sql.load(&mut conn1).expect("Error loading config"))
                .await?;

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
        Ok(info)
    }
}
