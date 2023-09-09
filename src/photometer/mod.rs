pub mod discovery;
pub mod payload;
pub mod transport;

use payload::Decoder;
use tracing::{debug, info};
use transport::serial;
use transport::udp;
use transport::{Sample, Transport};

use discovery::database;
use discovery::http;

use super::database::Pool;

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

pub async fn calibrate(pool: Pool, is_ref_phot: bool, is_dry_run: bool) {
    if is_ref_phot {
        let discoverer = database::Discoverer::new(&pool);
        let _info = discoverer.discover().await;
    } else {
        let discoverer = http::Discoverer::new("http://192.168.4.1/config");
        let _info = discoverer.discover().await;
    }
    if !is_dry_run {
        let mut transport = choose_transport_type(is_ref_phot).await;
        let mut decoder = choose_decoder_type(is_ref_phot);
        loop {
            let Sample(tstamp, raw_bytes) = transport.reading().await.expect("Reading task");
            //info!("{raw_bytes:?}");
            match decoder.decode(tstamp, &raw_bytes) {
                Ok((tstamp, payload)) => info!("{tstamp:?} {payload:?}"),
                Err(e) => debug!("{e:?}"),
            }
        }
    }
}
