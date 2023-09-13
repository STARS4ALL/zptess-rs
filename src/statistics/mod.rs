use super::database::Pool;
use super::Timestamp;
use crate::photometer::payload::info::Payload;
use tokio::sync::mpsc::Receiver;
use tracing::info;

pub async fn collect(_pool: Pool, mut chan: Receiver<(Timestamp, Payload)>, _is_ref_phot: bool) {
    while let Some(message) = chan.recv().await {
        info!("GOT = {:?}", message);
    }
}
