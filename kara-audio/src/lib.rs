use std::{sync::mpsc, thread};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use tracing::{debug, error};

use self::stream::{AudioStream, Event};

pub mod stream;

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
            volume: 3.0,
        }
    }
}

pub fn visualiser_stream(vis_settings: Config) -> mpsc::Sender<Event> {
    let audio_stream = AudioStream::init(vis_settings);
    let event_sender = audio_stream.get_event_sender();
    init_audio_sender(event_sender.clone());
    event_sender
}

pub fn init_audio_sender(event_sender: mpsc::Sender<Event>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        // Set up the input device and stream with the default input config.
        let device = host.default_input_device().unwrap();
        debug!("using audio device ({})", device.name().unwrap());
        let device_config = device.default_input_config().unwrap();

        let stream = match device_config.sample_format() {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &device_config.into(),
                    move |data, _: &_| handle_input_data_f32(data, event_sender.clone()),
                    err_fn,
                )
                .unwrap(),
            other => {
                error!("Unsupported sample format {:?}", other);
                panic!("Unsupported sample format {:?}", other);
            }
        };

        stream.play().unwrap();
        // parks the thread so stream.play() does not get dropped and stops
        thread::park();
    });
}

fn handle_input_data_f32(data: &[f32], sender: mpsc::Sender<Event>) {
    // sends the raw data to audio_stream via the event_sender
    sender.send(Event::SendData(data.to_vec())).unwrap();
}

fn err_fn(err: cpal::StreamError) {
    error!("an error occurred on stream: {}", err);
}
