mod logger;

use tracing::trace;

fn main() -> anyhow::Result<()> {
    logger::initialise_logger()?;
    let device_name: Option<&str> = None;
    let (stream_opts, _stream) = mic_rec::StreamOpts::new(device_name).unwrap();

    std::thread::spawn(move || {
        while let Ok(feed) = stream_opts.feed_receiver().recv() {
            trace!("{}", feed.len());
        }
    });

    Ok(())
}
