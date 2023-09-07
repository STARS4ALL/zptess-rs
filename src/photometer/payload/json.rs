// JSON parsing stuff
use super::info::Payload;
use serde_json;
use std::io::{Error, ErrorKind};

pub struct Decoder;

impl Decoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn decode(&self, line: &str) -> Result<Payload, Error> {
        match serde_json::from_str(line) {
            Ok(info) => Ok(Payload::Json(info)),
            Err(_) => Err(Error::new(ErrorKind::Other, "invalid payload")),
        }
    }
}
