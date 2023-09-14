use super::database::Pool;
use super::photometer::payload::info::Payload;
use super::Timestamp;
use std::collections::VecDeque;
use tokio::sync::mpsc::Receiver;
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
        queue.make_contiguous();
        result = true;
    }
    info!(
        "[{}] Waiting for enough samples, {} remaining",
        label,
        capacity - length
    );
    return result;
}

pub async fn collect_task(_pool: Pool, mut chan: Receiver<(Timestamp, Payload)>, capacity: usize) {
    let mut ref_queue = SampleQueue::with_capacity(capacity);
    let mut test_queue = SampleQueue::with_capacity(capacity);
    let mut meas1 = false;
    let mut meas2 = false;
    let mut meas = false;

    while let Some(message) = chan.recv().await {
        match message {
            (tstamp, Payload::Json(payload)) => {
                meas1 =
                    possibly_enqueue(&mut test_queue, tstamp, Payload::Json(payload), false, meas);
                meas = meas1 && meas2;
            }

            (tstamp, Payload::Cristogg(payload)) => {
                meas2 = possibly_enqueue(
                    &mut ref_queue,
                    tstamp,
                    Payload::Cristogg(payload),
                    true,
                    meas,
                );
                meas = meas1 && meas2;
            }
        }
    }
}
