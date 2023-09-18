use super::database::Pool;
use super::photometer::discovery::Info;
use super::photometer::payload::Payload;
use super::{Sample, Timestamp};
use anyhow::Result;
use statistical::{median, standard_deviation};
use std::collections::VecDeque;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, Instant};
use tracing::info;

pub mod auxiliary;
const LABEL: [&str; 2] = ["REF.", "TEST"];
const REF: usize = 0; // index into array
const TEST: usize = 1; // index into array

type SampleQueue = VecDeque<Sample>;

pub struct Statistics {
    info: [Info; 2],
    queue: [SampleQueue; 2],
    ready: [bool; 2],
    window: usize,
    global_ready: bool,
    round: usize,
    millis: u64, // Number of milliseconds to wait between rounds, usually 5000
    channel: Receiver<Sample>,
}

impl Statistics {
    fn new(
        window: usize,
        channel: Receiver<Sample>,
        _nrounds: usize,
        millis: u64,
        ref_info: Info,
        test_info: Info,
    ) -> Self {
        Self {
            info: [ref_info, test_info],
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
            self.info[idx].name,
        )
    }

    async fn one_round(&mut self, round: usize) {
        self.round = round;
        let begin = Instant::now();
        while let Some(message) = self.channel.recv().await {
            match message {
                (_, Payload::Json(_)) => {
                    self.possibly_enqueue(1, message, self.global_ready);
                    self.global_ready = self.ready[REF] && self.ready[TEST];
                }
                (_, Payload::Cristogg(_)) => {
                    self.possibly_enqueue(0, message, self.global_ready);
                    self.global_ready = self.ready[REF] && self.ready[TEST];
                }
            }
            if Instant::now().duration_since(begin) > Duration::from_millis(self.millis) {
                if self.global_ready {
                    self.queue[REF].make_contiguous();
                    self.queue[TEST].make_contiguous();
                    info!(
                        "================ Calculating statistics for round {} ================",
                        self.round
                    );
                    self.calculate(REF);
                    self.calculate(TEST);
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
            self.info[idx].name,
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
    ref_info: Info,
    test_info: Info,
) -> Result<()> {
    let mut state = Statistics::new(capacity, chan, nrounds, millis, ref_info, test_info);
    for i in 1..=nrounds {
        //one_round(i, &mut chan, &mut state).await;
        state.one_round(i).await;
    }
    info!("Statistics task finished");
    Ok(())
}
