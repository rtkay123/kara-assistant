use std::sync::atomic::{AtomicBool, Ordering};

use kara_recorder::{Samples, Stream};
use tracing::info;

#[tokio::main]
async fn main() {
    init_logger();

    let (audio_tx, audio_rx) = std::sync::mpsc::channel();
    let wake_word_detected = AtomicBool::new(false);

    let stream = Stream::new(audio_tx, None::<&str>);
    stream.start();

    loop {
        // listen for wake word
        if wake_word_detected.load(Ordering::Relaxed) {
            while let Ok(format) = audio_rx.recv() {
                match format {
                    Samples::F32(_data) => println!("hello f32"),
                    Samples::I16(_data) => println!("hello i16"),
                    Samples::U16(_data) => println!("hello u16"),
                }
            }
        }
    }
}

fn init_logger() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kara=debug,other_crate=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_ver = env!("CARGO_PKG_VERSION");
    info!(version = pkg_ver, "{} has started", pkg_name);
}
