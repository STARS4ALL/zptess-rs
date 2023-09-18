pub mod cristogg;
pub mod json;
use super::super::Timestamp;
use anyhow::Result;
use serde::Deserialize;

// --
// This is the decoded, new JSON format payload
#[derive(Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Json {
    pub udp: u32,
    pub rev: i8,
    pub name: String,
    pub freq: f32,
    pub mag: f32,
    pub tamb: f32,
    pub tsky: f32,
    pub wdBm: i16,
    pub ain: i16,
    pub ZP: f32,
}

// ---------------------------------------------------
// This is the old payload captured by the serial line
// ---------------------------------------------------
#[derive(Clone, Debug)]
pub struct Cristogg {
    pub freq: f32,
    pub tbox: f32,
    pub tsky: f32,
    pub zp: f32,
}

// ----------------------------------
// THIS IS THE COMBINED PAYLOAD UNION
//-----------------------------------
#[derive(Debug, Clone)]
pub enum Payload {
    Json(Json),
    Cristogg(Cristogg),
}

pub enum Decoder {
    Json(json::Decoder),
    Cristogg(cristogg::Decoder),
}

impl Decoder {
    pub fn decode(&mut self, tstamp: Timestamp, line: &str) -> Result<(Timestamp, Payload)> {
        match self {
            Decoder::Cristogg(p) => p.decode(tstamp, line),
            Decoder::Json(p) => p.decode(tstamp, line),
        }
    }
}
