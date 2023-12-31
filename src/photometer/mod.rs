pub mod discovery;
pub mod payload;
pub mod transport;
pub mod update;

use super::database::Pool;
use super::{Model, Sample};
use anyhow::Result;
use discovery::Info;
use payload::Decoder;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info};
use transport::serial;
use transport::udp;
use transport::{RawSample, Transport};

fn choose_decoder_type(is_ref_phot: bool) -> Decoder {
    if !is_ref_phot {
        Decoder::Json(payload::json::Decoder::new())
    } else {
        Decoder::Cristogg(payload::cristogg::Decoder::new())
    }
}

async fn choose_transport_type(is_ref_phot: bool) -> transport::Transport {
    if !is_ref_phot {
        Transport::Udp(udp::Transport::new(2255).await.expect("New UDP Transport"))
    } else {
        Transport::Serial(
            serial::Transport::new(9600)
                .await
                .expect("New serial Transport"),
        )
    }
}

pub async fn discover_test(_model: &Model) -> Result<Info> {
    discovery::http::Discoverer::new().discover().await
}

pub async fn discover_ref(pool: &Pool) -> Result<Info> {
    let discoverer = discovery::database::Discoverer::new(pool);
    discoverer.discover().await
}

pub async fn write_zero_point(_model: &Model, zp: f32) -> Result<()> {
    update::http::Updater::new().update_zp(zp).await?;
    info!("Updated Zero Point {:.02}", zp);
    Ok(())
}

pub async fn reading_task(chan: Sender<Sample>, is_ref_phot: bool) -> Result<()> {
    let mut transport = choose_transport_type(is_ref_phot).await;
    let mut decoder = choose_decoder_type(is_ref_phot);
    loop {
        let RawSample(tstamp, raw_bytes) = transport.reading().await?;
        //info!("{raw_bytes:?}");
        match decoder.decode(tstamp, &raw_bytes) {
            Ok((tstamp, payload)) => match chan.send((tstamp, payload)).await {
                Ok(_) => {}
                Err(_) => {
                    break;
                }
            },
            Err(e) => debug!("{e:?}"),
        }
    }
    if is_ref_phot {
        info!("Ref. Photometer task finished");
    } else {
        info!("Test Photometer task finished");
    }

    Ok(())
}
