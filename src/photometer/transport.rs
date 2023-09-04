pub mod udp {

    use super::super::payload::json::TESSPayload;
    use tokio::net::UdpSocket;
    use tracing::info;

    const BUF_SIZE: usize = 1024;

    async fn init() -> Result<UdpSocket, std::io::Error> {
        UdpSocket::bind("0.0.0.0:2255").await
    }

    pub async fn phot_task() {
        let socket = init().expect("binding UDP socket");
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

pub mod serial {

    // Serial Port Stuff
    use bytes::BytesMut;
    use futures::stream::StreamExt;
    use regex::Regex;
    use std::io;
    use tokio_serial::SerialPortBuilderExt;
    use tokio_serial::SerialStream;
    use tokio_util::codec::Decoder;

    #[cfg(unix)]
    const DEFAULT_TTY: &str = "/dev/ttyUSB0";

    // <fH 00430><tA +2945><tO +2439><mZ -0000>
    const PATTERN1: &str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
    const PATTERN2: &str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

    struct LineCodec;

    impl Decoder for LineCodec {
        type Item = String;
        type Error = std::io::Error;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            let newline = src.as_ref().iter().position(|b| *b == b'\n');
            if let Some(n) = newline {
                let line = src.split_to(n + 1);
                return match std::str::from_utf8(line.as_ref()) {
                    Ok(s) => Ok(Some(s.to_string())),
                    Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
                };
            }
            Ok(None)
        }
    }

    async fn serial_init() -> Result<SerialStream, std::io::Error> {
        let mut port = tokio_serial::new(DEFAULT_TTY, 9600)
            .open_native_async()
            .unwrap();
        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");
        Ok(port)
    }

    pub async fn phot_task() {
        let port = serial_init().await.expect("Opening serial");
        let mut reader = LineCodec.framed(port);
        loop {}
    }
}
