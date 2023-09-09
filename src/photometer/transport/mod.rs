pub mod serial;
pub mod udp;

use super::super::Timestamp;
use std::io::Error;

pub struct RawSample(pub Timestamp, pub String);

pub enum Transport {
    Serial(serial::Transport),
    Udp(udp::Transport),
}

impl Transport {
    pub async fn reading(&mut self) -> Result<RawSample, Error> {
        match self {
            Transport::Serial(t) => t.reading().await,
            Transport::Udp(t) => t.reading().await,
        }
    }
}
