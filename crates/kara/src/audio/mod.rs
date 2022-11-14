pub mod asr;
use crate::config::Configuration;
use crate::events::KaraEvent;
use crate::graphics::AudioEvent;
use crossbeam_channel::Sender;
use iced_winit::winit::event_loop::EventLoopProxy;
use mic_rec::StreamOpts;
use std::sync::{Arc, Mutex};

pub fn create_asr_sources() {
    let (tx, rx) = crossbeam_channel::bounded(1);
    tokio::spawn(async move {
        tx.send(5).unwrap();
    });
}

#[cfg(feature = "graphical")]
pub fn start_listening(
    stream_opts: StreamOpts,
    config: Arc<Mutex<Configuration>>,
    sample_rate: Option<f32>,
    event_loop: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
) -> Sender<AudioEvent> {
    let (visualiser_sender, event_receiver) = crossbeam_channel::unbounded();
    let visualiser_handle = visualiser_sender.clone();

    use ::asr::{sources::kara::LocalRecogniser, Transcibe};

    // blocking task that handles visualising
    use crate::graphics::visualise;
    visualise(config, event_receiver);

    // blocking task that listens for audio
    tokio::task::spawn_blocking(move || {
        let local_recogniser =
            LocalRecogniser::new("", sample_rate.unwrap_or_else(|| stream_opts.sample_rate()))
                .unwrap();
        let (tx, rx) = crossbeam_channel::unbounded();
        while let Ok(audio_buf) = stream_opts.audio_feed().recv() {
            let _transciption_data =
                audio_utils::resample_i16_mono(&audio_buf, stream_opts.channel_count());
            local_recogniser
                .transcribe(&_transciption_data, &tx)
                .unwrap();
            let _ = visualiser_sender.send(AudioEvent::SendData(audio_buf));

            if let Ok(ev) = rx.recv() {
                let proxy = event_loop.lock().unwrap();
                proxy
                    .send_event(if ev.finalised() {
                        KaraEvent::FinalisedSpeech(ev.transcription().to_string())
                    } else {
                        KaraEvent::ReadingSpeech(ev.transcription().to_string())
                    })
                    .unwrap();
            }
        }
    });

    visualiser_handle
}

pub fn get_audio_device_info(config: &Configuration) -> (Option<String>, Option<f32>) {
    match &config.audio {
        Some(audio) => (audio.input_device_name.clone(), audio.sample_rate),
        None => (None, None),
    }
}
