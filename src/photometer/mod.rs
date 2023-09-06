pub mod payload;
pub mod transport;

use std::io;
use tracing::info;

struct Photometer {
    is_ref_phot: bool,
    transport: transport::serial::Transport,
}

impl Photometer {
    pub async fn new(is_ref_phot: bool) -> Result<Self, io::Error> {
        Ok(Self {
            is_ref_phot,
            transport: transport::serial::Transport::new(9600).await?,
        })
    }

    pub async fn reading(&mut self) -> Result<String, io::Error> {
        self.transport.reading().await
    }
}

pub async fn task(is_ref_phot: bool) {
    let mut photometer = Photometer::new(is_ref_phot).await.expect("New Photometer");
    loop {
        let line = photometer.reading().await.expect("Reading task");
        info!("{line:?}");
    }
}

/*
Ok(Self {
            is_ref_phot,
            transport: transport::udp::Transport::new(2255).await?,
        })
*/
