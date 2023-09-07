use std::io::Error;

pub mod cristogg;
pub mod info;
pub mod json;

pub enum Decoder {
    Json(json::Decoder),
    Cristogg(cristogg::Decoder),
}

impl Decoder {
    pub fn decode(&self, line: &str) -> Result<info::Payload, Error> {
        match self {
            Decoder::Cristogg(p) => p.decode(line),
            Decoder::Json(p) => p.decode(line),
        }
    }
}
