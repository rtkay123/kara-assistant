pub mod errors;

pub use audio_utils::{convert_to_mono, split_channels};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat,
};
use tracing::{debug, error, info, trace};

use crate::errors::StreamOptsError;

pub struct StreamOpts {
    sample_rate: f32,
    audio_feed: crossbeam_channel::Receiver<Vec<f32>>,
    channel_count: u16,
}

pub struct Stream {
    stream: cpal::Stream,
}

type Result<T> = std::result::Result<T, StreamOptsError>;

impl StreamOpts {
    pub fn new(device_name: Option<impl AsRef<str>>) -> Result<(Self, Stream)> {
        let (raw_sender, raw_receiver) = crossbeam_channel::unbounded();
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
                    if let Err(e) = raw_sender.send(audio_utils::resample_f32(data)) {
                        error!("{e}")
                    }
                },
                error_callback,
                None,
            ),
            SampleFormat::U16 => device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    if let Err(e) = raw_sender.send(audio_utils::resample_f32(data)) {
                        error!("{e}")
                    }
                },
                error_callback,
                None,
            ),
            SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    if let Err(e) = raw_sender.send(data.to_owned()) {
                        error!("{e}")
                    }
                },
                error_callback,
                None,
            ),
            sample_format => {
                return Err(StreamOptsError::UnsupportedSampleFormat(
                    sample_format.to_string(),
                ))
            }
        }?;

        info!("stream is ready");
        Ok((
            StreamOpts {
                sample_rate: sample_rate as f32,
                audio_feed: raw_receiver,
                channel_count: channels,
            },
            Stream { stream },
        ))
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn audio_feed(&self) -> &crossbeam_channel::Receiver<Vec<f32>> {
        &self.audio_feed
    }
    pub fn channel_count(&self) -> u16 {
        self.channel_count
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
