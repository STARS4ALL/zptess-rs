pub mod serial;
pub mod udp;

use std::io;

pub enum Transport {
    Serial(serial::Transport),
    Udp(udp::Transport),
}

impl Transport {
    pub async fn reading(&mut self) -> Result<String, io::Error> {
        match self {
            Transport::Serial(t) => t.reading().await,
            Transport::Udp(t) => t.reading().await,
        }
    }
}
