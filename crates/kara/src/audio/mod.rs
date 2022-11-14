pub mod asr;
use crate::config::Configuration;
use crate::events::KaraEvent;
use crate::graphics::AudioEvent;
use ::asr::{
    sources::{kara::LocalRecogniser, Source, SpeechRecognisers},
    Transcibe,
};
use crossbeam_channel::{Receiver, Sender};
use iced_winit::winit::event_loop::EventLoopProxy;
use mic_rec::StreamOpts;
use std::sync::{Arc, Mutex};
use tracing::debug;

pub fn create_asr_sources(
    config: Arc<Mutex<Configuration>>,
    sample_rate: f32,
) -> Receiver<SpeechRecognisers> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    tokio::spawn(async move {
        let config_file = config.lock().unwrap();
        let mut speech_recognisers = SpeechRecognisers::new();

        for i in &config_file.speech_recognition.sources {
            debug!(source = i.to_string());
            let backend: Option<Box<dyn Transcibe>> = match &i {
                Source::Kara { model_path } => Some(Box::new(
                    LocalRecogniser::new(model_path, sample_rate).unwrap(),
                )),
                Source::IBMWatson {
                    api_key,
                    service_url,
                } => {
                    if api_key.is_empty() && service_url.is_empty() {
                        None
                    } else {
                        todo!("create watson instance");
                    }
                }
            };

            if let Some(backend) = backend {
                if i.to_string() == config_file.speech_recognition.default_source {
                    speech_recognisers.add_primary(backend);
                } else {
                    speech_recognisers.add(backend);
                }
            }
        }
        tx.send(speech_recognisers).unwrap();
    });
    rx
}

#[cfg(feature = "graphical")]
pub fn start_listening(
    stream_opts: StreamOpts,
    config: Arc<Mutex<Configuration>>,
    event_loop: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
    speech_recognisers: Receiver<SpeechRecognisers>,
) -> Sender<AudioEvent> {
    let (visualiser_sender, event_receiver) = crossbeam_channel::unbounded();
    let visualiser_handle = visualiser_sender.clone();

    // blocking task that handles visualising
    use crate::graphics::visualise;
    visualise(config, event_receiver);

    // blocking task that listens for audio
    tokio::task::spawn_blocking(move || {
        let (tx, rx) = crossbeam_channel::unbounded();
        if let Ok(recognisers) = speech_recognisers.recv() {
            while let Ok(audio_buf) = stream_opts.audio_feed().recv() {
                let _transciption_data =
                    audio_utils::resample_i16_mono(&audio_buf, stream_opts.channel_count());
                recognisers.speech_to_text(&_transciption_data, &tx);
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
