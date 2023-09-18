// Cristobal Garcia's old way to deliver readings
use super::super::super::Timestamp;
use super::payload::{Cristogg, Payload};
use regex::Regex;
use std::io::{Error, ErrorKind};

// <fH 00430><tA +2945><tO +2439><mZ -0000>
// <fm 15686><tA +3255><tO +3121><mZ -0000>
const HERTZ: &str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
const MILLIHERTZ: &str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

#[derive(Debug)]
pub struct Decoder {
    re: Vec<Regex>,
    sample: Option<(Timestamp, Cristogg)>, // prev sample to filter out duplicate readinngs
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            re: vec![
                Regex::new(HERTZ).expect("Failed pattern"),
                Regex::new(MILLIHERTZ).expect("Failed pattern"),
            ],
            sample: None,
        }
    }

    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload), Error> {
        for re in self.re.iter() {
            if let Some(result) = re.captures(line) {
                let cur_payload = Cristogg {
                    freq: result[1].trim().parse::<f32>().expect("Frequency") / 1000.0,
                    zp: result[4].trim().parse::<f32>().expect("ZP"),
                    tbox: result[2].trim().parse::<f32>().expect("Temp Box") / 100.0,
                    tsky: result[3].trim().parse::<f32>().expect("Temp Sky") / 100.0,
                };
                if let Some((t, p)) = self.filter(tstamp, cur_payload) {
                    return Ok((t, Payload::Cristogg(p)));
                } else {
                    return Err(Error::new(ErrorKind::Other, "duplicate Cristogg payload"));
                }
            } else {
                if line == "" {
                    return Err(Error::new(ErrorKind::Other, "empty Cristogg line"));
                }
            }
        }
        Err(Error::new(ErrorKind::Other, "invalid Cristogg payload"))
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
