// JSON parsing stuff
use super::super::super::Timestamp;
use super::{Json, Payload};
use anyhow::{bail, Result};
use serde_json;
use tracing::debug;

pub struct Decoder {
    sample: Option<(Timestamp, Json)>, // prev sample to filter out duplicate readinngs
}

// Ok((tstamp, Payload::Json(info)))

impl Decoder {
    pub fn new() -> Self {
        Self { sample: None }
    }

    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload)> {
        if let Ok(info) = serde_json::from_str(line) {
            if let Some((t, p)) = self.filter(tstamp, info) {
                Ok((t, Payload::Json(p)))
            } else {
                bail!("duplicate JSON payload")
            }
        } else {
            bail!("invalid JSON Decodification")
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
                debug!("Discarding duplicate JSON reading {cur_sample:?}");
                self.sample = Some(cur_sample); // duplicate reading
                None
            }
        } else {
            self.sample = Some(cur_sample); // bootstrapping the filter for the first time
            None
        }
    }
}
