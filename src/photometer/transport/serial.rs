// Serial Port Stuff

use super::RawSample;
use bytes::BytesMut;
use chrono::prelude::*;
use futures::stream::StreamExt;
use std::io;
use std::io::{Error, ErrorKind};
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyUSB0";

#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

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
                Err(_) => Err(Error::new(ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

// This is not needed for the time being */
impl Encoder<String> for LineCodec {
    type Error = io::Error;
    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}

type SerialReader = Framed<SerialStream, LineCodec>;

pub struct Transport {
    reader: SerialReader,
}

impl Transport {
    pub async fn new(baud: u32) -> Result<Self, io::Error> {
        let mut port = tokio_serial::new(DEFAULT_TTY, baud)
            .open_native_async()
            .unwrap();
        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");
        Ok(Self {
            reader: LineCodec.framed(port),
        })
    }

    pub async fn reading(&mut self) -> Result<RawSample, io::Error> {
        if let Some(line_result) = self.reader.next().await {
            let tstamp = Utc::now();
            let line = line_result.expect("Failed to read line");
            let line = line.trim();
            Ok(RawSample(tstamp, String::from(line)))
        } else {
            Err(Error::new(ErrorKind::Other, "No line_result"))
        }
    }
}
