// JSON parsing stuff
use crate::photometer::payload::info::PayloadInfo;
use serde_json;
use std::io::{Error, ErrorKind};

pub struct Payload;

impl Payload {
    pub fn new() -> Self {
        Self {}
    }

    pub fn decode(&self, line: &str) -> Result<PayloadInfo, Error> {
        match serde_json::from_str(line) {
            Ok(info) => Ok(PayloadInfo::Json(info)),
            Err(_) => Err(Error::new(ErrorKind::Other, "invalid payload")),
        }
    }
}
