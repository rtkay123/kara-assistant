use std::{
    sync::{mpsc, Arc},
    thread,
};

use coqui_stt::{Model, Stream};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample,
};
use tracing::{debug, error};

use self::stream::{AudioStream, Event};

pub mod stream;
const SAMPLE_RATE: u32 = 16000;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub buffering: usize,
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub resolution: usize,
    pub refresh_rate: usize,
    pub frequency_scale_range: [usize; 2],
    pub frequency_scale_amount: usize,
    pub density_reduction: usize,
    pub max_frequency: usize,
    pub volume: f32,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            buffering: 5,
            smoothing_size: 10,
            smoothing_amount: 5,
            resolution: 3000,
            refresh_rate: 60,
            frequency_scale_range: [50, 1000],
            frequency_scale_amount: 1,
            density_reduction: 5,
            //max_frequency: 20_000,
            max_frequency: 12_500,
            volume: 3.2,
        }
    }
}

pub async fn visualiser_stream(vis_settings: Config) -> mpsc::Sender<Event> {
    let audio_stream = AudioStream::init(vis_settings);
    let event_sender = audio_stream.get_event_sender();
    init_audio_sender(event_sender.clone()).await;
    event_sender
}

pub async fn init_audio_sender(event_sender: mpsc::Sender<Event>) {
    let mut stream =
        Stream::from_model(Arc::new(Model::new("kara-assets/kara-stt.tflite").unwrap())).unwrap();
    let (tx, rx) = mpsc::channel();
    tokio::spawn(async move {
        let host = cpal::default_host();
        // Set up the input device and stream with the default input config.
        let device = host.default_input_device().unwrap();
        debug!("using audio device ({})", device.name().unwrap());

        let mut config = device.default_input_config().unwrap();
        if config.channels() != 1 {
            let mut supported_configs_range = device.supported_input_configs().unwrap();
            config = match supported_configs_range.next() {
                Some(conf) => {
                    conf.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)) //16K from deepspeech
                }
                None => config,
            };
        }
        let stream = device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| {
                    send_to_visualiser(data, event_sender.clone());
                    tx.send(data.to_owned()).unwrap();
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();
        // parks the thread so stream.play() does not get dropped and stops
        thread::park();
    });
    tokio::spawn(async move {
        while let Ok(val) = rx.recv() {
            let val = val.iter().map(|f| f.to_i16()).collect::<Vec<_>>();
            stream.feed_audio(&val);
            if let Ok(val) = stream.intermediate_decode() {
                println!("{val}");
            }
        }
    });
}

fn send_to_visualiser(data: &[f32], sender: mpsc::Sender<Event>) {
    // sends the raw data to audio_stream via the event_sender
    sender.send(Event::SendData(data.to_vec())).unwrap();
}

fn err_fn(err: cpal::StreamError) {
    error!("an error occurred on stream: {}", err);
}
