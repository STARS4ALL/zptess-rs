pub mod auxiliary;
pub mod calibration;
pub mod dao;
pub mod readings;

use crate::Timestamp;
use statistical;
use std::collections::VecDeque;
use tracing::info;
// Re-exports for the submodules
pub use crate::database::Pool;
pub use crate::photometer::discovery::Info;
pub use crate::photometer::payload::Payload;
pub use crate::Sample;
// Re-exports for the other modules
pub use calibration::calibration_task;
pub use readings::reading_task;

type PayloadQueue = VecDeque<Payload>;
type TimestampQueue = VecDeque<Timestamp>;
pub type TimeWindow = (Timestamp, Timestamp); // t0, t1 time window

pub const LABEL: [&str; 2] = ["REF.", "TEST"];
pub const REF: usize = 0; // index into array
pub const TEST: usize = 1; // index into array

#[derive(Debug)]
pub struct CalibrationInfo {
    pub author: String,
    pub rounds: usize,
    pub offset: f32,
    pub zp_fict: f32,
}

impl Default for CalibrationInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CalibrationInfo {
    pub fn new() -> Self {
        Self {
            author: "".to_string(),
            rounds: 0,
            offset: 0.0,
            zp_fict: 0.0,
        }
    }
}

pub struct SamplesBuffer {
    label: &'static str,
    initial_size: usize,
    read_q: PayloadQueue,
    time_q: TimestampQueue,
    ready: bool,
    info: Info,
    zp_fict: f32,
}

impl SamplesBuffer {
    fn new(initial_size: usize, info: Info, label: &'static str, zp_fict: f32) -> Self {
        Self {
            read_q: PayloadQueue::with_capacity(initial_size),
            time_q: TimestampQueue::with_capacity(initial_size),
            ready: false,
            info,
            label,
            initial_size,
            zp_fict,
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
        let t0 = tstamps_slice.first().expect("t0 timestamp expected");
        let t1 = tstamps_slice.last().expect("t1 timestamp expected");
        let dur = (*t1 - *t0).to_std().expect("Duration Conversion").as_secs() as f32;
        tstamps_slice.len() as f32 / dur
    }

    fn median(&self) -> (f32, f32, f32, TimeWindow, f32) {
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
        let mag = auxiliary::magntude(freq, self.info.freq_offset, self.zp_fict);
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
            self.zp_fict,
        );
        (freq, stdev, mag, (t0, t1), dur as f32)
    }
}
