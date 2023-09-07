use std::io::Error;

pub mod cristogg;
pub mod info;
pub mod json;

pub enum Payload {
    Json(json::Payload),
    Cristogg(cristogg::Payload),
}

impl Payload {
    pub fn decode(&self, line: &str) -> Result<info::PayloadInfo, Error> {
        match self {
            Payload::Cristogg(p) => p.decode(line),
            Payload::Json(p) => p.decode(line),
        }
    }
}
