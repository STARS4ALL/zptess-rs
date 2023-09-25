use super::{Info, Payload, Pool, Sample, SamplesBuffer, LABEL, REF, TEST};
use crate::statistics::dao;
use anyhow::Result;
use std::cmp;
use tokio::sync::mpsc::Receiver;

pub struct Reading {
    refe: Option<SamplesBuffer>, // may not be present if reading the test photometer only
    test: Option<SamplesBuffer>, // may not be present if reading the ref photometer only
    channel: Receiver<Sample>,   // where to receive the samples from photometer tasks
}

impl Reading {
    fn new(
        window: usize,
        channel: Receiver<Sample>,
        ref_info: Option<Info>,
        test_info: Option<Info>,
        zp_fict: f32,
    ) -> Self {
        let rbuf = ref_info.map(|info| SamplesBuffer::new(window, info, LABEL[REF], zp_fict));
        let tbuf = test_info.map(|info| SamplesBuffer::new(window, info, LABEL[TEST], zp_fict));
        Self {
            channel,
            refe: rbuf,
            test: tbuf,
        }
    }

    async fn reading_both(&mut self) {
        let mut i: u8 = 0;
        while let Some(message) = self.channel.recv().await {
            match message {
                (tstamp, Payload::Json(reading)) => {
                    if let Some(ref mut queue) = self.test {
                        queue.enqueue(tstamp, Payload::Json(reading));
                    }
                }
                (tstamp, Payload::Cristogg(reading)) => {
                    if let Some(ref mut queue) = self.refe {
                        queue.enqueue(tstamp, Payload::Cristogg(reading));
                    }
                }
            }
            let test_queue = self.test.as_mut().unwrap();
            let refe_queue = self.refe.as_mut().unwrap();
            if test_queue.ready && refe_queue.ready {
                test_queue.make_contiguous();
                refe_queue.make_contiguous();
                let speed = refe_queue.speed() / test_queue.speed();
                let n = (if speed < 1.0 { 1.0 / speed } else { speed }).round() as u8;
                if i == 0 {
                    refe_queue.median();
                    test_queue.median();
                }
                i = (i + 1) % n;
            }
        }
    }

    async fn reading_single(&mut self) {
        let mut i: u8 = 0;
        let queue = if self.refe.is_some() {
            self.refe.as_mut().unwrap()
        } else {
            self.test.as_mut().unwrap()
        };
        while let Some(message) = self.channel.recv().await {
            let (tstamp, payload) = message;
            queue.enqueue(tstamp, payload);
            if queue.ready {
                queue.make_contiguous();
                let n = cmp::max((queue.speed()).round() as u8, 1);
                if i == 0 {
                    queue.median();
                }
                i = (i + 1) % n;
            }
        }
    }

    async fn reading(&mut self) {
        if self.refe.is_some() && self.test.is_some() {
            self.reading_both().await;
            return;
        }
        self.reading_single().await;
    }
}

pub async fn reading_task(
    pool: Pool,
    chan: Receiver<Sample>,
    capacity: usize,
    ref_info: Option<Info>,
    test_info: Option<Info>,
) -> Result<()> {
    let dao = dao::Dao::new(pool);
    let cal_info = dao.read_config().await?;
    let mut stats = Reading::new(capacity, chan, ref_info, test_info, cal_info.zp_fict);
    stats.reading().await;
    Ok(())
}
