pub mod payload;
pub mod transport;

use payload::Payload;
use tracing::info;
use transport::serial;
use transport::udp;
use transport::Transport;

fn choose_payload_type(is_ref_phot: bool) -> payload::Payload {
    let payload = if !is_ref_phot {
        Payload::Json(payload::json::Payload::new())
    } else {
        Payload::Cristogg(payload::cristogg::Payload::new())
    };
    payload
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

pub async fn task(is_ref_phot: bool) {
    let mut transport = choose_transport_type(is_ref_phot).await;

    let payload = choose_payload_type(is_ref_phot);
    loop {
        let raw_bytes = transport.reading().await.expect("Reading task");
        //info!("{raw_bytes:?}");
        match payload.decode(&raw_bytes) {
            Ok(payload_info) => info!("{payload_info:?}"),
            Err(_) => (),
        }
    }
}
