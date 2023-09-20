use super::database::Pool;
use super::photometer::discovery::Info;
use super::photometer::payload::Payload;
use super::{Sample, Timestamp};
use anyhow::Result;
use statistical;
use std::cmp;
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

pub struct SamplesBuffer {
    label: &'static str,
    initial_size: usize,
    read_q: PayloadQueue,
    time_q: TimestampQueue,
    ready: bool,
    info: Info,
}

impl SamplesBuffer {
    fn new(initial_size: usize, info: Info, label: &'static str) -> Self {
        Self {
            read_q: PayloadQueue::with_capacity(initial_size),
            time_q: TimestampQueue::with_capacity(initial_size),
            ready: false,
            info,
            label,
            initial_size,
        }
    }

    fn enqueue(&mut self, tstamp: Timestamp, payload: Payload) {
        let length = self.read_q.len();
        let capacity = self.read_q.capacity();
        if length < capacity {
            self.read_q.push_back(payload);
            self.time_q.push_back(tstamp);
            self.ready = false;
            info!(
                "[{}] {:9} Waiting for enough samples, {} remaining",
                self.label,
                self.info.name,
                capacity - length
            );
        } else {
            self.read_q.pop_front();
            self.time_q.pop_front();
            self.read_q.push_back(payload);
            self.time_q.push_back(tstamp);
            self.ready = true;
        }
    }

    fn possibly_enqueue(&mut self, tstamp: Timestamp, payload: Payload, accumulate: bool) {
        // let the read_q grow and grow so we can save all samples
        if accumulate {
            self.read_q.push_back(payload);
            self.time_q.push_back(tstamp);
            return;
        }
        self.enqueue(tstamp, payload);
    }

    fn make_contiguous(&mut self) {
        self.read_q.make_contiguous();
        self.time_q.make_contiguous();
    }

    fn speed(&self) -> f32 {
        let (tstamps_slice, _) = self.time_q.as_slices();
        let t0 = if let Some(t) = tstamps_slice.first() {
            t
        } else {
            panic!("Empty timestamp queue");
        };
        let t1 = if let Some(t) = tstamps_slice.last() {
            t
        } else {
            panic!("Empty timestamp queue");
        };
        let dur = (*t1 - *t0).to_std().expect("Duration Conversion").as_secs() as f32;
        tstamps_slice.len() as f32 / dur
    }

    fn median(&self) -> (f32, f32) {
        let from = self.read_q.len() - self.initial_size;
        let (readings_slice, _) = self.read_q.as_slices();
        let readings_slice = &readings_slice[from..];
        let (tstamps_slice, _) = self.time_q.as_slices();
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
        let freq = statistical::median(&freqs);
        let stdev = statistical::standard_deviation(&freqs, Some(freq));
        let mag = magntude(freq, self.info.freq_offset);
        info!(
            "{} {:9} ({}-{})[{:02}s][{}] median f = {:0.3} Hz, \u{03C3} = {:0.3} Hz, m = {:0.2} @ {:0.2}",
            self.label,
             self.info.name,
            t0.format("%H:%M:%S"),
            t1.format("%H:%M:%S"),
            dur,
            self.initial_size,
            freq,
            stdev,
            mag,
            ZERO_POINT_FICT,
        );
        (freq, mag)
    }
}

pub struct Calibration {
    refe: SamplesBuffer,
    test: SamplesBuffer,
    ready: bool, // Global redy flag computed from the two samples buffers
    round: usize,
    millis: u64, // Number of milliseconds to wait between rounds, usually 5000
    channel: Receiver<Sample>, // where to receive the sampels form photometer tasks
    freqs: [Vec<f32>; 2], // central estimator of frequencies (currently median)
    mags: [Vec<f32>; 2], // central estimator of frequencies (currently median)
}

impl Calibration {
    fn new(
        window: usize,
        channel: Receiver<Sample>,
        nrounds: usize,
        millis: u64,
        ref_info: Info,
        test_info: Info,
    ) -> Self {
        Self {
            refe: SamplesBuffer::new(window, ref_info, LABEL[REF]),
            test: SamplesBuffer::new(window, test_info, LABEL[TEST]),
            ready: false,
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

    fn accumulate(&mut self, idx: usize, freq: f32, mag: f32) {
        self.freqs[idx].push(freq);
        self.mags[idx].push(mag);
    }

    async fn one_round(&mut self, round: usize) {
        self.round = round;
        let begin = Instant::now();
        while let Some(message) = self.channel.recv().await {
            match message {
                (tstamp, Payload::Json(reading)) => {
                    self.test
                        .possibly_enqueue(tstamp, Payload::Json(reading), self.ready);
                    self.ready = self.refe.ready && self.test.ready;
                }

                (tstamp, Payload::Cristogg(reading)) => {
                    self.refe
                        .possibly_enqueue(tstamp, Payload::Cristogg(reading), self.ready);
                    self.ready = self.refe.ready && self.test.ready;
                }
            }
            if Instant::now().duration_since(begin) > Duration::from_millis(self.millis) {
                if self.ready {
                    self.refe.make_contiguous();
                    self.test.make_contiguous();
                    info!(
                        "========================= Calculating statistics for round {} =========================",
                        self.round
                    );
                    let (r_freq, r_mag) = self.refe.median();
                    let (t_freq, t_mag) = self.test.median();
                    let mag_diff = r_mag - t_mag;
                    let zp = self.refe.info.zp + mag_diff;
                    info!("ROUND {:02}: New ZP = {:0.2} = \u{0394}(ref-test) Mag ({:0.2}) + ZP Abs ({:0.2})",
                        self.round, zp, mag_diff, self.refe.info.zp);
                    self.accumulate(REF, r_freq, r_mag);
                    self.accumulate(TEST, t_freq, t_mag);
                    break;
                }
            }
        }
    }
}

pub async fn calibration_task(
    _pool: Pool,
    chan: Receiver<Sample>,
    capacity: usize,
    nrounds: usize,
    millis: u64,
    ref_info: Info,
    test_info: Info,
) -> Result<()> {
    let mut calib = Calibration::new(capacity, chan, nrounds, millis, ref_info, test_info);
    for i in 1..=nrounds {
        calib.one_round(i).await;
    }
    info!("Calibration task finished");
    Ok(())
}

pub struct Reading {
    refe: SamplesBuffer,
    test: SamplesBuffer,
    channel: Receiver<Sample>, // where to receive the sampels form photometer tasks
}

impl Reading {
    fn new(window: usize, channel: Receiver<Sample>, ref_info: Info, test_info: Info) -> Self {
        Self {
            channel,
            refe: SamplesBuffer::new(window, ref_info, LABEL[REF]),
            test: SamplesBuffer::new(window, test_info, LABEL[TEST]),
        }
    }

    async fn reading_both(&mut self) {
        let mut i: u8 = 0;
        while let Some(message) = self.channel.recv().await {
            match message {
                (tstamp, Payload::Json(reading)) => {
                    self.test.enqueue(tstamp, Payload::Json(reading));
                }
                (tstamp, Payload::Cristogg(reading)) => {
                    self.refe.enqueue(tstamp, Payload::Cristogg(reading));
                }
            }
            if self.test.ready && self.refe.ready {
                self.refe.make_contiguous();
                self.test.make_contiguous();
                // Adjust the tracing output speed to the slowest rate
                let speed = self.test.speed() / self.refe.speed();
                let n = (if speed < 1.0 { 1.0 / speed } else { speed }).round() as u8;
                if i == 0 {
                    self.refe.median();
                    self.test.median();
                }
                i = (i + 1) % n;
            }
        }
    }

    async fn reading_single(&mut self, is_ref: bool) {
        let mut i: u8 = 0;
        while let Some(message) = self.channel.recv().await {
            match message {
                (tstamp, Payload::Json(reading)) => {
                    self.test.enqueue(tstamp, Payload::Json(reading));
                    if !is_ref && self.test.ready {
                        self.test.make_contiguous();
                        let n = cmp::max((self.test.speed()).round() as u8, 1);
                        if i == 0 {
                            self.test.median();
                        }
                        i = (i + 1) % n;
                    }
                }
                (tstamp, Payload::Cristogg(reading)) => {
                    self.refe.enqueue(tstamp, Payload::Cristogg(reading));
                    if is_ref && self.refe.ready {
                        self.refe.make_contiguous();
                        let n = cmp::max((self.refe.speed()).round() as u8, 1);
                        if i == 0 {
                            self.refe.median();
                        }
                        i = (i + 1) % n;
                    }
                }
            }
        }
    }
}

pub async fn reading_task(
    _pool: Pool,
    chan: Receiver<Sample>,
    capacity: usize,
    ref_info: Info,
    test_info: Info,
) -> Result<()> {
    let mut stats = Reading::new(capacity, chan, ref_info, test_info);
    stats.reading_single(false).await;
    Ok(())
}
