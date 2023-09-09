// Cristobal Garcia's old way to deliver readings
use super::super::super::Timestamp;
use super::info::{Cristogg, Payload};
use regex::Regex;
use std::io::{Error, ErrorKind};

// <fH 00430><tA +2945><tO +2439><mZ -0000>
// <fm 15686><tA +3255><tO +3121><mZ -0000>
const HERTZ: &str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
const MILLIHERTZ: &str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

#[derive(Debug)]
pub struct Decoder {
    re: Vec<Regex>,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            re: vec![
                Regex::new(HERTZ).expect("Failed pattern"),
                Regex::new(MILLIHERTZ).expect("Failed pattern"),
            ],
        }
    }

    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload), Error> {
        for re in self.re.iter() {
            if let Some(result) = re.captures(line) {
                return Ok((
                    tstamp,
                    Payload::Cristogg(Cristogg {
                        freq: result[1].trim().parse::<f32>().expect("Frequency") / 1000.0,
                        zp: result[4].trim().parse::<f32>().expect("ZP"),
                        tbox: result[2].trim().parse::<f32>().expect("Temp Box") / 100.0,
                        tsky: result[3].trim().parse::<f32>().expect("Temp Sky") / 100.0,
                    }),
                ));
            } else {
                if line == "" {
                    return Err(Error::new(ErrorKind::Other, "empty line"));
                }
            }
        }
        Err(Error::new(ErrorKind::Other, "invalid cristogg payload"))
    }
}
