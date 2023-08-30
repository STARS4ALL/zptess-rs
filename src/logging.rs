
use tracing;
use tracing_subscriber;

pub fn init(level: tracing::Level) {
    let format = tracing_subscriber::fmt::format()
        .with_level(true) // include levels in formatted output
        .with_target(true) // don't include targets
        .with_thread_ids(false) // don't include the thread ID of the current thread
        .with_thread_names(false) // include the name of the current thread
        .compact(); // use the `Compact` formatting style.
    tracing_subscriber::fmt().event_format(format).with_max_level(level).init();
}

