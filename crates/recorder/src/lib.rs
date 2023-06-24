use std::sync::{mpsc::Sender, Arc};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use log::{debug, error, trace};

pub struct Stream {
    pub device_name: Result<String, cpal::DeviceNameError>,
    stream: cpal::Stream,
}

pub enum Samples {
    F32(Arc<[f32]>),
    I16(Arc<[i16]>),
    U16(Arc<[u16]>),
}

impl Stream {
    pub fn start(&self) {
        self.stream.play().unwrap();
    }
    pub fn pause(&self) {
        self.stream.pause().unwrap();
    }

    pub fn new(audio_sender: Sender<Samples>, device_name: Option<impl AsRef<str>>) -> Self {
        let host = cpal::default_host();

        let device = match device_name {
            None => host.default_input_device(),
            Some(device_name) => host
                .input_devices()
                .unwrap()
                .find(|x| x.name().map(|y| y == device_name.as_ref()).unwrap_or(false))
                .or(host.default_input_device()),
        }
        .expect("no input device");

        let config = device.default_input_config().unwrap();

        let error_callback = move |err| {
            error!("{err}");
        };

        if let Ok(name) = device.name() {
            trace!("Using audio device: {}", name);
        }

        let sample_rate = config.sample_rate().0;

        let channels = config.channels();
        trace!("Using {} channels @{}Hz", channels, sample_rate);

        let stream = match config.sample_format() {
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let _ = audio_sender.send(Samples::I16(data.into()));
                    //
                },
                error_callback,
                None,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    let _ = audio_sender.send(Samples::U16(data.into()));
                    //
                },
                error_callback,
                None,
            ),
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let _ = audio_sender.send(Samples::F32(data.into()));
                    //
                },
                error_callback,
                None,
            ),
            _sample_format => {
                todo!("handle audio stream error")
                //
            }
        }
        .unwrap();

        debug!("stream is ready");
        Stream {
            device_name: device.name(),
            stream,
        }
    }
}
