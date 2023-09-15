use super::database::Pool;
use super::photometer::payload::info::Payload;
use super::Timestamp;
use anyhow::Result;
use chrono;

use statistical::{mean, median, standard_deviation};
use std::collections::VecDeque;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, Instant};
use tracing::info;
const LABEL: [&str; 2] = ["REF.", "TEST"];

type Sample = (Timestamp, Payload, String);
type SampleQueue = VecDeque<Sample>;

pub struct Statistics {
    queue: [SampleQueue; 2],
    ready: [bool; 2],
    window: usize,
    global_ready: bool,
    round: usize,
    millis: u64, // Number of milliseconds to wait between rounds, usually 5000
    channel: Receiver<Sample>,
}

impl Statistics {
    fn new(window: usize, channel: Receiver<Sample>, _nrounds: usize, millis: u64) -> Self {
        Self {
            queue: [
                SampleQueue::with_capacity(window),
                SampleQueue::with_capacity(window),
            ],
            window,
            ready: [false, false],
            global_ready: false,
            round: 1,
            millis,  // Milliseconds to wait between rounds, usually 5000
            channel, // Take ownership of the receiver end of the channel
        }
    }

    fn calculate(&mut self, idx: usize) {
        let from = self.queue[idx].len() - self.window;
        let (slice, _) = self.queue[idx].as_mut_slices();
        let slice = &slice[from..];
        let name = slice[0].2.clone();
        let tstamps: Vec<Timestamp> = slice.iter().map(|tup| tup.0).collect();
        let freqs: Vec<f32> = slice
            .iter()
            .map(|tup| match tup.1.clone() {
                Payload::Json(p) => p.freq,
                Payload::Cristogg(p) => p.freq,
            })
            .collect();
        let t0 = tstamps[0];
        let t1 = tstamps[tstamps.len() - 1];
        let dur = (t1 - t0).to_std().expect("Duration Conversion").as_secs();
        let central = median(&freqs);
        let stdev = standard_deviation(&freqs, Some(central));
        info!(
            "{0} {7:9} ({1}-{2})[{3:02}s][{4}] median f = {5}, \u{03C3} = {6}",
            LABEL[idx],
            t0.format("%H:%M:%S"),
            t1.format("%H:%M:%S"),
            dur,
            self.window,
            central,
            stdev,
            name
        )
    }

    async fn one_round(&mut self, round: usize) {
        self.round = round;
        let begin = Instant::now();
        while let Some(message) = self.channel.recv().await {
            match message {
                (_, Payload::Json(_), _) => {
                    self.possibly_enqueue(1, message, self.global_ready);
                    self.global_ready = self.ready[0] && self.ready[1];
                }
                (_, Payload::Cristogg(_), _) => {
                    self.possibly_enqueue(0, message, self.global_ready);
                    self.global_ready = self.ready[0] && self.ready[1];
                }
            }
            if Instant::now().duration_since(begin) > Duration::from_millis(self.millis) {
                if self.global_ready {
                    self.queue[0].make_contiguous();
                    self.queue[1].make_contiguous();
                    info!(
                        "================ Calculating statistics for round {} ================",
                        self.round
                    );
                    self.calculate(0);
                    self.calculate(1);
                    break;
                }
            }
        }
    }

    fn possibly_enqueue(&mut self, idx: usize, sample: Sample, is_measuring: bool) {
        // let the queue grow and grow so we can save all samples
        if is_measuring {
            self.queue[idx].push_back(sample);
            return;
        }
        let name = sample.2.clone();
        let length = self.queue[idx].len();
        let capacity = self.queue[idx].capacity();
        if length < capacity {
            self.queue[idx].push_back(sample);
            self.ready[idx] = false;
        } else {
            self.queue[idx].pop_front();
            self.queue[idx].push_back(sample);
            self.ready[idx] = true;
        }
        info!(
            "[{}] {:9} Waiting for enough samples, {} remaining",
            LABEL[idx],
            name,
            capacity - length
        );
    }
}

pub async fn collect_task(
    _pool: Pool,
    chan: Receiver<Sample>,
    capacity: usize,
    nrounds: usize,
    millis: u64,
) -> Result<()> {
    let mut state = Statistics::new(capacity, chan, nrounds, millis);
    for i in 1..=nrounds {
        //one_round(i, &mut chan, &mut state).await;
        state.one_round(i).await;
    }
    info!("Statistics task finished");
    Ok(())
}
