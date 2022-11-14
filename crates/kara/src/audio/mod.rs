pub mod asr;
use crate::config::Configuration;
use crate::graphics::AudioEvent;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use mic_rec::StreamOpts;
use std::sync::{Arc, Mutex};

#[cfg(feature = "graphical")]
pub fn start_listening(
    stream_opts: StreamOpts,
    visualiser_sender: Sender<AudioEvent>,
    config: Arc<Mutex<Configuration>>,
    event_receiver: Receiver<AudioEvent>,
) {
    use crate::graphics::visualise;

    visualise(config, event_receiver);
    tokio::task::spawn_blocking(move || {
        while let Ok(audio_buf) = stream_opts.audio_feed().recv() {
            let _transciption_data =
                audio_utils::resample_i16_mono(&audio_buf, stream_opts.channel_count());
            let _ = visualiser_sender.send(AudioEvent::SendData(audio_buf));
        }
    });
}

pub fn get_audio_device_info(config: &Configuration) -> (Option<String>, Option<f32>) {
    match &config.audio {
        Some(audio) => (audio.input_device_name.clone(), audio.sample_rate),
        None => (None, None),
    }
}
