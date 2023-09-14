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

fn possibly_enqueue(queue: &mut SampleQueue, t: Timestamp, p: Payload, is_ref: bool) {
    let label = if is_ref { REF_LABEL } else { TEST_LABEL };
    if queue.len() == queue.capacity() {
        queue.pop_front();
    } else {
        info!(
            "[{}] Waiting for enough samples, {} remaining",
            label,
            queue.capacity() - queue.len()
        );
    }
    queue.push_back((t, p));
}

pub async fn collect_task(_pool: Pool, mut chan: Receiver<(Timestamp, Payload)>, capacity: usize) {
    let mut ref_queue = SampleQueue::with_capacity(capacity);
    let mut test_queue = SampleQueue::with_capacity(capacity);

    while let Some(message) = chan.recv().await {
        match message {
            (tstamp, Payload::Json(payload)) => {
                possibly_enqueue(&mut test_queue, tstamp, Payload::Json(payload), false);
            }

            (tstamp, Payload::Cristogg(payload)) => {
                possibly_enqueue(&mut ref_queue, tstamp, Payload::Cristogg(payload), true);
            }
        }
    }
}
