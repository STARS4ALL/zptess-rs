pub mod database;
pub mod http;

#[derive(Debug)]
pub struct Info {
    pub model: String,
    pub name: String,
    pub mac: String,
    pub firmware: String,
    pub sensor: String,
    pub zp: f32,
    pub freq_offset: f32,
}

impl Info {
    fn new() -> Self {
        Self {
            model: "TESS-W".into(),
            name: "".into(),
            mac: "".into(),
            firmware: "".into(),
            sensor: "TSL237".into(),
            zp: 0.0,
            freq_offset: 0.0,
        }
    }
}
