pub mod udp {

    use bytes::BytesMut;
    use std::io;
    use tokio::net::UdpSocket;

    const BUF_SIZE: usize = 1024;
    const ANY_ADDR: &str = "0.0.0.0";

    pub struct Transport {
        socket: UdpSocket,
        buffer: BytesMut,
    }

    impl Transport {
        pub async fn new(port: u16) -> Result<Self, io::Error> {
            let mut endpoint = String::from(ANY_ADDR);
            endpoint.push(':');
            endpoint.push_str(&port.to_string());
            Ok(Self {
                socket: UdpSocket::bind(endpoint).await?,
                buffer: BytesMut::with_capacity(BUF_SIZE),
            })
        }

        pub async fn reading(&mut self) -> Result<String, io::Error> {
            let (len, _src) = self.socket.recv_buf_from(&mut self.buffer).await?;
            let s = String::from(
                std::str::from_utf8(&self.buffer[..len])
                    .expect("invalid UTF-8")
                    .trim(),
            );
            self.buffer.clear();
            Ok(s)
        }
    }
}

pub mod serial {

    // Serial Port Stuff
    use bytes::BytesMut;
    use futures::stream::StreamExt;
    use regex::Regex;
    use std::io;
    use std::io::{Error, ErrorKind};
    use tokio_serial::SerialPortBuilderExt;
    use tokio_serial::SerialStream;
    use tokio_util::codec::{Decoder, Encoder, Framed};
    use tracing::info;

    #[cfg(unix)]
    const DEFAULT_TTY: &str = "/dev/ttyUSB0";

    #[cfg(windows)]
    const DEFAULT_TTY: &str = "COM1";

    // <fH 00430><tA +2945><tO +2439><mZ -0000>
    const PATTERN1: &str = r"^<fH([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";
    const PATTERN2: &str = r"^<fm([ +]\d{5})><tA ([+-]\d{4})><tO ([+-]\d{4})><mZ ([+-]\d{4})>";

    struct LineCodec;

    impl Decoder for LineCodec {
        type Item = String;
        type Error = io::Error;

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

    /*
    impl Encoder<String> for LineCodec {
        type Error = io::Error;
        fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    */

    pub struct Transport {
        reader: Framed<SerialStream, LineCodec>,
    }

    impl Transport {
        pub async fn new(baud: u32) -> Result<Self, io::Error> {
            let mut port = tokio_serial::new(DEFAULT_TTY, baud)
                .open_native_async()
                .unwrap();
            #[cfg(unix)]
            port.set_exclusive(false)
                .expect("Unable to set serial port exclusive to false");
            Ok(Transport {
                reader: LineCodec.framed(port),
            })
        }

        pub async fn reading(&mut self) -> Result<String, io::Error> {
            if let Some(line_result) = self.reader.next().await {
                let line = line_result.expect("Failed to read line");
                let line = line.trim();
                Ok(String::from(line))
            } else {
                Err(Error::new(ErrorKind::Other, "oh no!"))
            }
        }
    }
    /*
    impl Decoder for Transport {
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
    /*
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
    */
    */
}
