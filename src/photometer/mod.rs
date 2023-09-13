pub mod discovery;
pub mod payload;
pub mod transport;
pub mod update;

use super::database::Pool;
use anyhow::Result;

use discovery::{database, Info};
use payload::Decoder;
use tracing::{debug, info};
use transport::serial;
use transport::udp;
use transport::{RawSample, Transport};

fn choose_decoder_type(is_ref_phot: bool) -> Decoder {
    let decoder = if !is_ref_phot {
        Decoder::Json(payload::json::Decoder::new())
    } else {
        Decoder::Cristogg(payload::cristogg::Decoder::new())
    };
    decoder
}

async fn choose_transport_type(is_ref_phot: bool) -> transport::Transport {
    let transport = if !is_ref_phot {
        Transport::Udp(udp::Transport::new(2255).await.expect("New UDP Transport"))
    } else {
        Transport::Serial(
            serial::Transport::new(9600)
                .await
                .expect("New serial Transport"),
        )
    };
    transport
}

pub async fn discover() -> Result<Info> {
    discovery::http::Discoverer::new().discover().await
}

pub async fn write_zero_point(zp: f32) -> Result<()> {
    update::http::Updater::new().update_zp(zp).await?;
    info!("Updated Zero Point {:.02}", zp);
    Ok(())
}

pub async fn calibrate(pool: Pool, is_ref_phot: bool) {
    if is_ref_phot {
        let discoverer = database::Discoverer::new(&pool);
        let _info = discoverer.discover().await;
    } else {
        let discoverer = discovery::http::Discoverer::new();
        let _info = discoverer.discover().await;
    }

    let mut transport = choose_transport_type(is_ref_phot).await;
    let mut decoder = choose_decoder_type(is_ref_phot);
    loop {
        let RawSample(tstamp, raw_bytes) = transport.reading().await.expect("Reading task");
        //info!("{raw_bytes:?}");
        match decoder.decode(tstamp, &raw_bytes) {
            Ok((tstamp, payload)) => info!("{tstamp:?} {payload:?}"),
            Err(e) => debug!("{e:?}"),
        }
    }
}
