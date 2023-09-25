use super::{CalibrationInfo, Pool};
use crate::database::{models::Config, Db};
use anyhow::Result;
use diesel::prelude::*;
use tokio::task;
use tracing::{debug, error};

pub struct Dao {
    pool: Pool,
}

impl Dao {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    pub async fn read_config(&self) -> Result<CalibrationInfo> {
        use crate::database::schema::config_t::dsl::*;
        let sql = config_t
            .filter(section.eq("calibration"))
            .select(Config::as_select());

        debug!("{:?}", diesel::debug_query::<Db, _>(&sql).to_string());
        let mut conn1 = self.pool.get()?;
        let results =
            task::spawn_blocking(move || sql.load(&mut conn1).expect("Error loading config"))
                .await?;

        let mut info = CalibrationInfo::new();
        for item in results.iter() {
            match item.property.as_str() {
                "author" => info.author = item.value.clone(),
                "rounds" => info.rounds = item.value.clone().parse::<usize>()?,
                "offset" => info.offset = item.value.clone().parse::<f32>()?,
                "zp_fict" => info.zp_fict = item.value.clone().parse::<f32>()?,
                &_ => error!("{}", item.property),
            }
        }
        //info!("{info:#?}");
        Ok(info)
    }
}
