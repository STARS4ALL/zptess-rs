use super::database::Pool;
use super::photometer::payload::info::Payload;
use super::Timestamp;
use anyhow::Result;
use std::collections::VecDeque;
use tokio::sync::mpsc::Receiver;
use tokio::time::{sleep, Duration};
use tracing::info;

const REF_LABEL: &str = "REF.";
const TEST_LABEL: &str = "TEST";

type Sample = (Timestamp, Payload);
type SampleQueue = VecDeque<Sample>;

pub struct Statistics {
    ref_q: SampleQueue,
    test_q: SampleQueue,
    window: usize,
    ref_flag: bool,
    test_flag: bool,
    glob_flag: bool,
    nrounds: usize,
    round: usize,
    channel: Receiver<Sample>,
}

impl Statistics {
    fn new(window: usize, mut channel: Receiver<Sample>, nrounds: usize) -> Self {
        Self {
            ref_q: SampleQueue::with_capacity(window),
            test_q: SampleQueue::with_capacity(window),
            window,
            ref_flag: false,
            test_flag: false,
            glob_flag: false,
            nrounds,
            round: 1,
            channel, // Take ownership of the receiver end of the channel
        }
    }

    fn calculate(&self) {
        info!("==== Calculating statistics for round {} ====", self.round);
        info!("REF  Q LEN = {:?}", self.ref_q.len());
        info!("TEST Q LEN = {:?}", self.test_q.len());
    }

    async fn one_round(&mut self, round: usize) {
        self.round = round;
        while let Some(message) = self.channel.recv().await {
            match message {
                (tstamp, Payload::Json(payload)) => {
                    self.possibly_enqueue_test(tstamp, Payload::Json(payload), self.glob_flag);
                    self.glob_flag = self.test_flag && self.ref_flag;
                }

                (tstamp, Payload::Cristogg(payload)) => {
                    self.possibly_enqueue_ref(tstamp, Payload::Cristogg(payload), self.glob_flag);
                    self.glob_flag = self.test_flag && self.ref_flag;
                }
            }
            if self.glob_flag {
                self.ref_q.make_contiguous();
                self.test_q.make_contiguous();
                self.calculate();
                break;
            }
        }
    }

    fn possibly_enqueue_test(&mut self, t: Timestamp, p: Payload, is_measuring: bool) {
        let label = TEST_LABEL;
        // let the queue grow and grow so we can save all samples
        if is_measuring {
            self.test_q.push_back((t, p));
            return;
        }
        let length = self.test_q.len();
        let capacity = self.test_q.capacity();
        if length < capacity {
            self.test_q.push_back((t, p));
            self.test_flag = false;
        } else {
            self.test_q.pop_front();
            self.test_q.push_back((t, p));
            self.test_flag = true;
        }
        info!(
            "[{}] Waiting for enough samples, {} remaining",
            label,
            capacity - length
        );
    }

    fn possibly_enqueue_ref(&mut self, t: Timestamp, p: Payload, is_measuring: bool) {
        let label = REF_LABEL;
        // let the queue grow and grow so we can save all samples
        if is_measuring {
            self.ref_q.push_back((t, p));
            return;
        }
        let length = self.ref_q.len();
        let capacity = self.ref_q.capacity();

        if length < capacity {
            self.ref_q.push_back((t, p));
            self.ref_flag = false;
        } else {
            self.ref_q.pop_front();
            self.ref_q.push_back((t, p));
            self.ref_flag = true;
        }
        info!(
            "[{}] Waiting for enough samples, {} remaining",
            label,
            capacity - length
        );
    }
}

pub async fn collect_task(
    _pool: Pool,
    chan: Receiver<Sample>,
    capacity: usize,
    n: usize,
) -> Result<()> {
    let mut state = Statistics::new(capacity, chan, n);
    for i in 1..=n {
        sleep(Duration::from_millis(5000)).await;
        //one_round(i, &mut chan, &mut state).await;
        state.one_round(i).await;
        info!("==== Calibration round {i} done ====");
    }
    info!("Statistics task finished");
    Ok(())
}
