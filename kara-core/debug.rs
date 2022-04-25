use std::path::PathBuf;

use tracing::{info, Level};
use tracing_subscriber::{
    filter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
};

pub fn start_logger() -> tracing_appender::non_blocking::WorkerGuard {
    let filter = filter::Targets::new().with_target("kara", Level::TRACE);
    let file_appender = tracing_appender::rolling::daily(log_dir(), "kara.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(filter)
        .init();
    info!(
        "{} {} has started",
        env!("CARGO_BIN_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    guard
}

fn log_dir() -> PathBuf {
    let mut cache_dir = dirs::cache_dir().expect("could not find cache dir");
    cache_dir.push("kara");
    cache_dir
}
