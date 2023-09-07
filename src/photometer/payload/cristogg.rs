// Cristobal Garcia's old way to deliver readings
use crate::photometer::payload::info::{Cristogg, Payload};
use regex::Regex;
use std::io::{Error, ErrorKind};

// <fH 00430><tA +2945><tO +2439><mZ -0000>
// <fm 15686><tA +3255><tO +3121><mZ -0000>
const PATTERN1: &'static str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
const PATTERN2: &'static str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

#[derive(Debug)]
pub struct Decoder {
    re: Vec<Regex>,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            re: vec![
                Regex::new(PATTERN1).expect("Failed pattern"),
                Regex::new(PATTERN2).expect("Failed pattern"),
            ],
        }
    }

    pub fn decode(&self, line: &str) -> Result<Payload, Error> {
        for re in self.re.iter() {
            if let Some(result) = re.captures(line) {
                tracing::info!("{:?}", result);
                return Ok(Payload::Cristogg(Cristogg {
                    freq: result[1].trim().parse::<f32>().expect("Frequency") / 1000.0,
                    zp: result[4].trim().parse::<f32>().expect("ZP"),
                    tbox: result[2].trim().parse::<f32>().expect("Temp Box") / 100.0,
                    tsky: result[3].trim().parse::<f32>().expect("Temp Sky") / 100.0,
                }));
            }
        }
        Err(Error::new(ErrorKind::Other, "invalid payload"))
    }
}
