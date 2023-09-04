pub mod udp {

    use super::super::payload::json::TESSPayload;
    use tokio::net::UdpSocket;
    use tracing::info;

    const BUF_SIZE: usize = 1024;

    pub async fn phot_task() {
        let socket = UdpSocket::bind("0.0.0.0:2255")
            .await
            .expect("binding UDP socket");
        tracing::info!("Creado el socket");
        loop {
            let mut buf = [0; BUF_SIZE];
            let (amt, _src) = socket.recv_from(&mut buf).await.unwrap();
            // Redeclare `buf` as slice of the received data.
            let buf = &mut buf[..amt];
            let s = std::str::from_utf8(buf).expect("invalid UTF-8").trim();
            let reading = TESSPayload::new(s);
            info!("{reading:?}");
        }
    }
}
