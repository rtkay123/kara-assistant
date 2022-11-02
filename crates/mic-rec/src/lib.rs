mod audio_utils;
pub mod errors;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use tracing::{debug, error, trace};

use crate::errors::StreamOptsError;

pub struct StreamOpts {
    sample_rate: f32,
    audio_receiver: crossbeam_channel::Receiver<Vec<i16>>,
}

pub struct Stream {
    stream: cpal::Stream,
}

type Result<T> = std::result::Result<T, StreamOptsError>;

impl StreamOpts {
    pub fn new(device_name: Option<impl AsRef<str>>) -> Result<(Self, Stream)> {
        let (audio_sender, audio_receiver) = crossbeam_channel::unbounded();
        trace!("setting up audio device");
        let host = cpal::default_host();

        let device = match device_name {
            None => host.default_input_device(),
            Some(device_name) => host
                .input_devices()?
                .find(|x| x.name().map(|y| y == device_name.as_ref()).unwrap_or(false)),
        }
        .ok_or(StreamOptsError::NoInputDevice)?;

        let config = device.default_input_config()?;

        let error_callback = move |err| {
            error!("{err}");
        };

        if let Ok(name) = device.name() {
            debug!(name = name, "audio input device");
        }

        let sample_rate = config.sample_rate().0;

        let channels = config.channels();
        debug!(channels = channels, sample_rate = sample_rate);

        let stream = match config.sample_format() {
            SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    if let Err(e) = audio_sender.send(audio_utils::resample(data, channels)) {
                        error!("{e}")
                    }
                },
                error_callback,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    if let Err(e) = audio_sender.send(audio_utils::resample(data, channels)) {
                        error!("{e}")
                    }
                },
                error_callback,
            ),
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    if let Err(e) = audio_sender.send(audio_utils::resample(data, channels)) {
                        error!("{e}")
                    }
                },
                error_callback,
            ),
        }?;

        trace!("stream created");
        Ok((
            StreamOpts {
                sample_rate: sample_rate as f32,
                audio_receiver,
            },
            Stream { stream },
        ))
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn feed_receiver(&self) -> &crossbeam_channel::Receiver<Vec<i16>> {
        &self.audio_receiver
    }
}
impl Stream {
    pub fn start_stream(&self) -> Result<()> {
        trace!("starting audio stream");
        Ok(self.stream.play()?)
    }
}

#[cfg(test)]
mod tests;
