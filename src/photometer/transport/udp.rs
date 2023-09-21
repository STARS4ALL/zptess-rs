use super::RawSample;
use bytes::BytesMut;
use chrono::prelude::*;
use std::io;
use tokio::net::UdpSocket;

const BUF_SIZE: usize = 256;
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

    pub async fn reading(&mut self) -> Result<RawSample, io::Error> {
        let (len, _src) = self.socket.recv_buf_from(&mut self.buffer).await?;
        let tstamp = Utc::now();
        let s = String::from(
            std::str::from_utf8(&self.buffer[..len])
                .expect("invalid UTF-8")
                .trim(),
        );
        self.buffer.clear();
        Ok(RawSample(tstamp, s))
    }
}
