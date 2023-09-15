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

fn possibly_enqueue(
    queue: &mut SampleQueue,
    t: Timestamp,
    p: Payload,
    is_ref: bool,
    is_measuring: bool,
) -> bool {
    let label = if is_ref { REF_LABEL } else { TEST_LABEL };

    // let the queue grow and grow so we can save all samples
    if is_measuring {
        queue.push_back((t, p));
        return true;
    }
    let length = queue.len();
    let capacity = queue.capacity();
    let mut result = false;

    if length < capacity {
        queue.push_back((t, p));
    } else {
        queue.pop_front();
        queue.push_back((t, p));

        result = true;
    }
    info!(
        "[{}] Waiting for enough samples, {} remaining",
        label,
        capacity - length
    );
    return result;
}

pub struct GlobalState {
    pub ref_q: SampleQueue,
    pub test_q: SampleQueue,
    pub ref_flag: bool,
    pub test_flag: bool,
    pub glob_flag: bool,
}

impl GlobalState {
    fn new(capacity: usize) -> Self {
        Self {
            ref_q: SampleQueue::with_capacity(capacity),
            test_q: SampleQueue::with_capacity(capacity),
            ref_flag: false,
            test_flag: false,
            glob_flag: false,
        }
    }
}

pub async fn one_round(round: usize, chan: &mut Receiver<Sample>, mut state: &mut GlobalState) {
    while let Some(message) = chan.recv().await {
        match message {
            (tstamp, Payload::Json(payload)) => {
                state.test_flag = possibly_enqueue(
                    &mut state.test_q,
                    tstamp,
                    Payload::Json(payload),
                    false,
                    state.glob_flag,
                );
                state.glob_flag = state.test_flag && state.ref_flag;
            }

            (tstamp, Payload::Cristogg(payload)) => {
                state.ref_flag = possibly_enqueue(
                    &mut state.ref_q,
                    tstamp,
                    Payload::Cristogg(payload),
                    true,
                    state.glob_flag,
                );
                state.glob_flag = state.test_flag && state.ref_flag;
            }
        }
        if state.glob_flag {
            state.ref_q.make_contiguous();
            state.test_q.make_contiguous();
            calculate_stats(&mut state, round);
            break;
        }
    }
}

fn calculate_stats(state: &mut GlobalState, round: usize) {
    info!("==== Calculating statistics for round {round} ====");
    //info!("REF  SLICES = {:?}", state.ref_q.as_slices());
    //info!("TEST SLICES = {:?}", state.test_q.as_slices());

    info!("REF  Q LEN = {:?}", state.ref_q.len());
    info!("TEST Q LEN = {:?}", state.test_q.len());
}

pub async fn collect_task(
    _pool: Pool,
    mut chan: Receiver<Sample>,
    capacity: usize,
    n: usize,
) -> Result<()> {
    let mut state = GlobalState::new(capacity);
    for i in 1..=n {
        sleep(Duration::from_millis(5000)).await;
        one_round(i, &mut chan, &mut state).await;
        info!("==== Calibration round {i} done ====");
    }
    info!("Statistics task finished");
    Ok(())
}
