pub mod cristogg;
pub mod json;
pub mod payload;

use super::super::Timestamp;
use payload::Payload;
use std::io::Error;

pub enum Decoder {
    Json(json::Decoder),
    Cristogg(cristogg::Decoder),
}

impl Decoder {
    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload), Error> {
        match self {
            Decoder::Cristogg(p) => p.decode(tstamp, line),
            Decoder::Json(p) => p.decode(tstamp, line),
        }
    }
}
