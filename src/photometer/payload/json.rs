// JSON parsing stuff
use super::super::super::Timestamp;
use super::info::{Json, Payload};
use serde_json;
use std::io::{Error, ErrorKind};

pub struct Decoder {
    sample: Option<(Timestamp, Json)>, // prev sample to filter out duplicate readinngs
}

// Ok((tstamp, Payload::Json(info)))

impl Decoder {
    pub fn new() -> Self {
        Self { sample: None }
    }

    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload), Error> {
        if let Ok(info) = serde_json::from_str(line) {
            if let Some((t, p)) = self.filter(tstamp, info) {
                Ok((t, Payload::Json(p)))
            } else {
                Err(Error::new(ErrorKind::Other, "duplicate JSON payload"))
            }
        } else {
            Err(Error::new(ErrorKind::Other, "invalid JSON decodification"))
        }
    }

    // Filter duplicated readings
    fn filter(&mut self, tstamp: Timestamp, reading: Json) -> Option<(Timestamp, Json)> {
        let cur_sample = (tstamp, reading);
        if let Some(prev_sample) = &self.sample {
            if prev_sample.1.udp != cur_sample.1.udp {
                let result = self.sample.clone(); // Can't apply move semantics behind mut self
                self.sample = Some(cur_sample);
                result
            } else {
                self.sample = Some(cur_sample); // duplicate reading
                None
            }
        } else {
            self.sample = Some(cur_sample); // bootstrapping the filter for the first time
            None
        }
    }
}
