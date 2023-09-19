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
const ZERO_POINT_FICT: f32 = 20.5;

type PayloadQueue = VecDeque<Payload>;
type TimestampQueue = VecDeque<Timestamp>;

fn magntude(freq: f32, freq_offset: f32) -> f32 {
    ZERO_POINT_FICT - 2.5 * (freq - freq_offset).log10()
}

pub struct Statistics {
    info: [Info; 2],
    read_q: [PayloadQueue; 2],
    time_q: [TimestampQueue; 2],
    ready: [bool; 2],
    window: usize,
    global_ready: bool,
    round: usize,
    millis: u64, // Number of milliseconds to wait between rounds, usually 5000
    channel: Receiver<Sample>, // where to receive the sampels form photometer tasks
    freqs: [Vec<f32>; 2], // central estimator of frequencies (currently median)
    mags: [Vec<f32>; 2], // central estimator of frequencies (currently median)
}

impl Statistics {
    fn new(
        window: usize,
        channel: Receiver<Sample>,
        nrounds: usize,
        millis: u64,
        ref_info: Info,
        test_info: Info,
    ) -> Self {
        Self {
            info: [ref_info, test_info],
            read_q: [
                PayloadQueue::with_capacity(window), // Ref
                PayloadQueue::with_capacity(window), // Test
            ],
            time_q: [
                TimestampQueue::with_capacity(window),
                TimestampQueue::with_capacity(window),
            ],
            window,
            ready: [false, false],
            global_ready: false,
            round: 1,
            millis,  // Milliseconds to wait between rounds, usually 5000
            channel, // Take ownership of the receiver end of the channel
            freqs: [
                Vec::<f32>::with_capacity(nrounds),
                Vec::<f32>::with_capacity(nrounds),
            ],
            mags: [
                Vec::<f32>::with_capacity(nrounds),
                Vec::<f32>::with_capacity(nrounds),
            ],
        }
    }

    fn calculate(&self, idx: usize) -> (f32, f32){
        let from = self.read_q[idx].len() - self.window;
        let (readings_slice, _) = self.read_q[idx].as_slices();
        let readings_slice = &readings_slice[from..];
        let (tstamps_slice, _) = self.time_q[idx].as_slices();
        let tstamps_slice = &tstamps_slice[from..];
        let freqs: Vec<f32> = readings_slice
            .iter()
            .map(|x| match x.clone() {
                Payload::Json(payload) => payload.freq,
                Payload::Cristogg(payload) => payload.freq,
            })
            .collect();

        let t0 = tstamps_slice[0];
        let t1 = tstamps_slice[tstamps_slice.len() - 1];
        let dur = (t1 - t0).to_std().expect("Duration Conversion").as_secs();
        let freq = median(&freqs);
        let stdev = standard_deviation(&freqs, Some(freq));
        let mag = magntude(freq, self.info[idx].freq_offset);
        info!(
            "{} {:9} ({}-{})[{:02}s][{}] median f = {:0.3} Hz, \u{03C3} = {:0.3} Hz, m = {:0.2} @ {:0.2}",
            LABEL[idx],
             self.info[idx].name,
            t0.format("%H:%M:%S"),
            t1.format("%H:%M:%S"),
            dur,
            self.window,
            freq,
            stdev,
           
            mag,
            ZERO_POINT_FICT,
        );
        (freq, mag)
    }

    fn possibly_enqueue(
        &mut self,
        idx: usize,
        tstamp: Timestamp,
        payload: Payload,
        is_measuring: bool,
    ) {
        // let the read_q grow and grow so we can save all samples
        if is_measuring {
            self.read_q[idx].push_back(payload);
            self.time_q[idx].push_back(tstamp);
            return;
        }
        let length = self.read_q[idx].len();
        let capacity = self.read_q[idx].capacity();
        if length < capacity {
            self.read_q[idx].push_back(payload);
            self.time_q[idx].push_back(tstamp);
            self.ready[idx] = false;
        } else {
            self.read_q[idx].pop_front();
            self.time_q[idx].pop_front();
            self.read_q[idx].push_back(payload);
            self.time_q[idx].push_back(tstamp);
            self.read_q[idx].make_contiguous();
            self.time_q[idx].make_contiguous();
            self.ready[idx] = true;
        }
        info!(
            "[{}] {:9} Waiting for enough samples, {} remaining",
            LABEL[idx],
            self.info[idx].name,
            capacity - length
        );
    }

    async fn one_round(&mut self, round: usize) {
        self.round = round;
        let begin = Instant::now();
        while let Some(message) = self.channel.recv().await {
            match message {
                (tstamp, Payload::Json(reading)) => {
                    self.possibly_enqueue(TEST, tstamp, Payload::Json(reading), self.global_ready);
                    self.global_ready = self.ready[REF] && self.ready[TEST];
                }

                (tstamp, Payload::Cristogg(reading)) => {
                    self.possibly_enqueue(
                        REF,
                        tstamp,
                        Payload::Cristogg(reading),
                        self.global_ready,
                    );
                    self.global_ready = self.ready[REF] && self.ready[TEST];
                }
            }
            if Instant::now().duration_since(begin) > Duration::from_millis(self.millis) {
                if self.global_ready {
                    self.read_q[REF].make_contiguous();
                    self.read_q[TEST].make_contiguous();
                    info!(
                        "========================= Calculating statistics for round {} =========================",
                        self.round
                    );
                    self.calculate(REF);
                    self.calculate(TEST);
                    break;
                }
            }
        }
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
