pub mod asr;
use crate::{
    audio::asr::{get_remote_model, try_default_location},
    config::Configuration,
    events::KaraEvent,
    graphics::AudioEvent,
};
use ::asr::{
    sources::{kara::LocalRecogniser, Source, SpeechRecognisers},
    Transcibe,
};
use crossbeam_channel::{Receiver, Sender};
use iced_winit::winit::event_loop::EventLoopProxy;
use mic_rec::StreamOpts;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, span, trace, warn, Level};

pub fn create_asr_sources(
    config: Arc<Mutex<Configuration>>,
    sample_rate: f32,
    event_loop: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
) -> (Receiver<SpeechRecognisers>, Receiver<LocalRecogniser>) {
    let (tx, rx) = crossbeam_channel::bounded(1);

    let (tx_local_model, rx_local_model) = crossbeam_channel::bounded(1);
    tokio::spawn(async move {
        let config_file = config.lock().expect("failed to acquire config lock");
        let recogniser_count = &config_file.speech_recognition.sources.len();
        debug!(
            available_recognisers = recogniser_count,
            "creating recognisers"
        );

        let mut speech_recognisers = SpeechRecognisers::new();

        for i in &config_file.speech_recognition.sources {
            debug!(current_source = i.to_string());
            let backend: Option<Box<dyn Transcibe>> = match &i {
                Source::Kara {
                    model_path,
                    fallback_url,
                } => {
                    let span = span!(Level::TRACE, "kara");
                    let _enter = span.enter();
                    trace!("configuring local recogniser");
                    match LocalRecogniser::new(model_path, sample_rate) {
                        Ok(model) => Some(Box::new(model)),
                        Err(e) => {
                            error!(path = model_path.display().to_string(), "{e}");
                            if i.to_string() == config_file.speech_recognition.default_source {
                                let model_path = if model_path.as_os_str().is_empty() {
                                    res_def::model_path()
                                } else {
                                    model_path.to_owned()
                                };
                                match try_default_location(&model_path, sample_rate) {
                                    Ok(model) => {
                                        let _ = tx_local_model.send(model);
                                    }
                                    Err(e) => {
                                        error!("{e}");
                                        tokio::spawn(get_remote_model(
                                            Arc::clone(&event_loop),
                                            tx_local_model.clone(),
                                            fallback_url.clone(),
                                            model_path.clone(),
                                            sample_rate,
                                        ));
                                    }
                                }
                            }
                            None
                        }
                    }
                }
                Source::IBMWatson {
                    api_key,
                    service_url,
                } => {
                    let span = span!(Level::TRACE, "ibm_watson");
                    let _enter = span.enter();
                    trace!("configuring ibm watson");
                    if api_key.is_empty() && service_url.is_empty() {
                        warn!(source = "IBM Watson", "missing [api_key] or [service_url]");
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
        let _ = tx.send(speech_recognisers);
    });
    (rx, rx_local_model)
}

#[cfg(feature = "graphical")]
pub fn start_listening(
    stream_opts: StreamOpts,
    config: Arc<Mutex<Configuration>>,
    event_loop: Arc<Mutex<EventLoopProxy<KaraEvent>>>,
    speech_recognisers: (Receiver<SpeechRecognisers>, Receiver<LocalRecogniser>),
) -> Sender<AudioEvent> {
    let (visualiser_sender, event_receiver) = crossbeam_channel::unbounded();
    let visualiser_handle = visualiser_sender.clone();

    // blocking task that handles visualising
    use crate::graphics::visualise;
    visualise(config, event_receiver);

    // blocking task that listens for audio
    tokio::task::spawn_blocking(move || {
        let (tx, rx) = crossbeam_channel::unbounded();

        let (speech_recognisers, local_recogniser) = speech_recognisers;
        if let Ok(mut recognisers) = speech_recognisers.recv() {
            while let Ok(audio_buf) = stream_opts.audio_feed().recv() {
                let transciption_data =
                    audio_utils::resample_i16_mono(&audio_buf, stream_opts.channel_count());
                if recognisers.valid() {
                    trace!("valid");
                    if let Err(e) = recognisers.speech_to_text(&transciption_data, &tx) {
                        error!("{e}");
                    }
                    if let Ok(ev) = rx.recv() {
                        let proxy = event_loop.lock().unwrap();
                        let _ = proxy.send_event(if ev.finalised() {
                            KaraEvent::FinalisedSpeech(ev.transcription().to_string())
                        } else {
                            KaraEvent::ReadingSpeech(ev.transcription().to_string())
                        });
                    }
                } else {
                    trace!("not valid");
                    if let Ok(rec) = local_recogniser.try_recv() {
                        recognisers.add_primary(Box::new(rec));
                    }
                }
                let _ = visualiser_sender.send(AudioEvent::SendData(audio_buf));
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
