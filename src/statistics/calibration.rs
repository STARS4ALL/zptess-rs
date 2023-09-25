use super::{
    Info, Payload, Pool, Sample, SamplesBuffer, TimeWindow, Timestamp, LABEL, REF, TEST,
    ZERO_POINT_FICT,
};
use crate::statistics::auxiliary;
use anyhow::Result;
use chrono::SecondsFormat;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, Instant};
use tracing::info;

pub struct Calibration {
    session: Timestamp,
    refe: SamplesBuffer,
    test: SamplesBuffer,
    ready: bool, // Global redy flag computed from the two samples buffers
    round: usize,
    millis: u64, // Number of milliseconds to wait between rounds, usually 5000
    channel: Receiver<Sample>, // where to receive the sampels form photometer tasks
    freqs: [Vec<f32>; 2], // median frequency for the current round
    stdevs: [Vec<f32>; 2], //  Stanbdard deviations for the current round
    mags: [Vec<f32>; 2], //  magnitude for the current round
    zps: Vec<f32>, //  zero point for the current round
    tstamps: [Vec<TimeWindow>; 2], //  timestamps limits for the current round
    durs: [Vec<f32>; 2], //  Turations for the crrent round
}

impl Calibration {
    fn new(
        window: usize,
        session: Timestamp,
        channel: Receiver<Sample>,
        nrounds: usize,
        millis: u64,
        ref_info: Info,
        test_info: Info,
    ) -> Self {
        Self {
            session,
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
            stdevs: [
                Vec::<f32>::with_capacity(nrounds),
                Vec::<f32>::with_capacity(nrounds),
            ],
            mags: [
                Vec::<f32>::with_capacity(nrounds),
                Vec::<f32>::with_capacity(nrounds),
            ],
            tstamps: [
                Vec::<TimeWindow>::with_capacity(nrounds),
                Vec::<TimeWindow>::with_capacity(nrounds),
            ],
            durs: [
                Vec::<f32>::with_capacity(nrounds),
                Vec::<f32>::with_capacity(nrounds),
            ],
            zps: Vec::<f32>::with_capacity(nrounds),
        }
    }

    fn accumulate(&mut self, idx: usize, freq: f32, stdev: f32, mag: f32, w: TimeWindow, dur: f32) {
        self.freqs[idx].push(freq);
        self.stdevs[idx].push(stdev);
        self.mags[idx].push(mag);
        self.tstamps[idx].push(w);
        self.durs[idx].push(dur);
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
            if Instant::now().duration_since(begin) > Duration::from_millis(self.millis)
                && self.ready
            {
                self.refe.make_contiguous();
                self.test.make_contiguous();
                info!(
                    "========================= Calculating statistics for round {} =========================",
                    self.round
                );
                let (r_freq, r_stdev, r_mag, r_win, r_dur) = self.refe.median();
                let (t_freq, t_stdev, t_mag, t_win, t_dur) = self.test.median();
                let mag_diff = r_mag - t_mag;
                let zp = auxiliary::round(self.refe.info.zp + mag_diff, 2);
                info!("ROUND {:02}: New ZP = {:0.2} = \u{0394}(ref-test) Mag ({:0.2}) + ZP Abs ({:0.2})",
                    self.round, zp, mag_diff, self.refe.info.zp);
                self.accumulate(REF, r_freq, r_stdev, r_mag, r_win, r_dur);
                self.accumulate(TEST, t_freq, t_stdev, t_mag, t_win, t_dur);
                self.zps.push(zp);
                break;
            }
        }
    }

    fn summary(&self) -> f32 {
        let offset_zp = 0.0; // A capón aqui
        info!("########################################################################");
        let best_zp = auxiliary::mode_or_median(&self.zps, 2, "ZP");
        let final_zp = best_zp + offset_zp;
        let best_ref_freq = auxiliary::mode_or_median(&self.freqs[REF], 3, "REF. Best freq.");
        let best_test_freq = auxiliary::mode_or_median(&self.freqs[TEST], 3, "TEST Best freq.");
        let best_ref_mag = auxiliary::magntude(best_ref_freq, 0.0, ZERO_POINT_FICT);
        let best_test_mag = auxiliary::magntude(best_test_freq, 0.0, ZERO_POINT_FICT);
        info!(
            "Session = {}",
            self.session.to_rfc3339_opts(SecondsFormat::Millis, true)
        );
        info!("Best ZP List is        {:?}", self.zps);
        info!("Best REF. Freq List is {:?}", self.freqs[REF]);
        info!("Best TEST Freq List is {:?}", self.freqs[TEST]);
        info!(
            "REF. Best Freq. = {:0.3} Hz, Mag = {:0.2}, Diff {:0.2}",
            best_ref_freq, best_ref_mag, 0.0
        );
        info!(
            "TEST. Best Freq. = {:0.3} Hz, Mag = {:0.2}, Diff {:0.2}",
            best_test_freq, best_test_mag, 0.0
        );
        info!(
            "Final TEST ZP ({:0.2}) = Best ZP ({:0.2}) + ZP offset ({:0.2})",
            final_zp, best_zp, offset_zp
        );
        info!(
            "Old TEST ZP = {:0.2}, NEW TEST ZP = {:0.2}",
            self.test.info.zp, final_zp
        );
        info!("########################################################################");
        final_zp
    }
}

pub async fn calibration_task(
    _pool: Pool,
    session: Timestamp,
    chan: Receiver<Sample>,
    capacity: usize,
    nrounds: usize,
    millis: u64,
    ref_info: Info,
    test_info: Info,
) -> Result<f32> {
    let mut calib = Calibration::new(
        capacity, session, chan, nrounds, millis, ref_info, test_info,
    );
    for i in 1..=nrounds {
        calib.one_round(i).await;
    }
    let zp = calib.summary();
    info!("Calibration task finished");
    Ok(zp)
}
