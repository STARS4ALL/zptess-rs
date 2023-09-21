use std::path::PathBuf;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, Registry};

pub fn init(level: Level, console: bool, log_path: Option<PathBuf>) -> Vec<WorkerGuard> {
    let mut guards = Vec::new();

    let layer1 = if console {
        let layer1 = fmt::layer()
            .with_level(true) // include levels in formatted output
            .with_target(false) // don't include targets
            .with_thread_ids(false) // don't include the thread ID of the current thread
            .with_thread_names(false) // include the name of the current thread
            .compact() // use the `Compact` formatting style.
            .with_writer(std::io::stdout.with_max_level(level));
        Some(layer1)
    } else {
        None
    };

    let layer2 = if let Some(path) = log_path {
        let file_writer = tracing_appender::rolling::never("", path);
        let (file_writer, guard) = tracing_appender::non_blocking(file_writer);
        guards.push(guard);
        let layer2 = fmt::layer()
            .with_level(true) // include levels in formatted output
            .with_target(false) // don't include targets
            .with_thread_ids(false) // don't include the thread ID of the current thread
            .with_thread_names(false) // include the name of the current thread
            .compact() // use the `Compact` formatting style.
            .with_writer(file_writer.with_max_level(level));
        Some(layer2)
    } else {
        None
    };
    let subscriber = Registry::default().with(layer1).with(layer2);
    tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");
    guards
}
