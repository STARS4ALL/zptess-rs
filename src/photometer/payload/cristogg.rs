// Cristobal Garcia's old way to deliver readings
use super::super::super::Timestamp;
use super::{Cristogg, Payload};
use anyhow::{bail, Result};
use regex::Regex;
use tracing::debug;

const HERTZ: &str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
const MILLIHERTZ: &str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

#[derive(Debug)]
pub struct Decoder {
    re: [Regex; 2],
    sample: Option<(Timestamp, Cristogg)>, // prev sample to filter out duplicate readinngs
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            re: [
                Regex::new(HERTZ).expect("Failed pattern"),
                Regex::new(MILLIHERTZ).expect("Failed pattern"),
            ],
            sample: None,
        }
    }

    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload)> {
        if let Some(cristogg) = self.matches(line) {
            if let Some((t, p)) = self.filter(tstamp, cristogg) {
                return Ok((t, Payload::Cristogg(p)));
            }
        } else {
            bail!("Empty Cristogg line")
        }
        bail!("Error decoding Cristogg payload")
    }

    pub fn matches(&self, line: &str) -> Option<Cristogg> {
        for re in self.re.iter() {
            if let Some(result) = re.captures(line) {
                let cristogg = Cristogg {
                    freq: result[1].trim().parse::<f32>().expect("Frequency") / 1000.0,
                    zp: result[4].trim().parse::<f32>().expect("ZP"),
                    tbox: result[2].trim().parse::<f32>().expect("Temp Box") / 100.0,
                    tsky: result[3].trim().parse::<f32>().expect("Temp Sky") / 100.0,
                };
                return Some(cristogg);
            }
        }
        None
    }

    // Filter duplicated readings
    fn filter(&mut self, tstamp: Timestamp, reading: Cristogg) -> Option<(Timestamp, Cristogg)> {
        let cur_sample = (tstamp, reading);
        if let Some(prev_sample) = &self.sample {
            if prev_sample.1.freq == cur_sample.1.freq
                && prev_sample.1.tsky == cur_sample.1.tsky
                && prev_sample.1.tbox == cur_sample.1.tbox
            {
                // duplicate reading, update and signal we have nothing
                debug!("Discarding duplicate Cristogg reading {cur_sample:?}");
                self.sample = Some(cur_sample);
                None
            } else {
                let result = self.sample.clone(); // Can't apply move semantics behind mut self
                self.sample = Some(cur_sample);
                result
            }
        } else {
            self.sample = Some(cur_sample); // bootstrapping the filter for the first time
            None
        }
    }
}
