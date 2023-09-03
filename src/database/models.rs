use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::database::schema::config_t)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Config {
    pub section: String,
    pub property: String,
    pub value: String,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::database::schema::batch_t)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Batch {
    pub begin_tstamp: String,
    pub end_tstamp: Option<String>,
    pub email_sent: Option<i32>,
    pub comment: Option<String>,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::database::views::summary_v)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SummaryView {
    pub session: String,
    pub role: String,
    pub calibration: Option<String>,
    pub calversion: Option<String>,
    pub model: Option<String>,
    pub name: Option<String>,
    pub mac: Option<String>,
    pub firmware: Option<String>,
    pub sensor: Option<String>,
    pub prev_zp: Option<f32>,
    pub author: Option<String>,
    pub nrounds: Option<i32>,
    pub offset: Option<f32>,
    pub upd_flag: Option<i32>,
    pub zero_point: Option<f32>,
    pub zero_point_method: Option<String>,
    pub test_freq: Option<f32>,
    pub test_freq_method: Option<String>,
    pub test_mag: Option<f32>,
    pub ref_freq: Option<f32>,
    pub ref_freq_method: Option<String>,
    pub ref_mag: Option<f32>,
    pub mag_diff: Option<f32>,
    pub raw_zero_point: Option<f32>,
    pub filter: Option<String>,
    pub plug: Option<String>,
    pub box_: Option<String>,
    pub collector: Option<String>,
    pub comment: Option<String>,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::database::views::rounds_v)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RoundsView {
    pub session: String,
    pub round: i32,
    pub role: String,
    pub begin_tstamp: Option<String>,
    pub end_tstamp: Option<String>,
    pub central: Option<String>,
    pub freq: Option<f32>,
    pub stddev: Option<f32>,
    pub mag: Option<f32>,
    pub zp_fict: Option<f32>,
    pub zero_point: Option<f32>,
    pub nsamples: Option<i32>,
    pub duration: Option<f32>,
    pub model: Option<String>,
    pub name: Option<String>,
    pub mac: Option<String>,
    pub nrounds: Option<i32>,
    pub upd_flag: Option<i32>,
}
