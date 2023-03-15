use thiserror::Error;

#[derive(Error, Debug)]
pub enum StreamOptsError {
    #[error("input device with name `{0}` is not available")]
    InvalidDeviceName(String),
    #[error("system does not support audio devices")]
    NoAudioDeviceSupport(#[from] cpal::DevicesError),
    #[error("failed to locate input device")]
    NoInputDevice,
    #[error("missing default input stream format")]
    StreamConfig(#[from] cpal::DefaultStreamConfigError),
    #[error("failed to build the stream")]
    Disconnect(#[from] cpal::BuildStreamError),
    #[error("failed to start the stream")]
    PlayStream(#[from] cpal::PlayStreamError),
    #[error("failed to send the audio feed")]
    AudioFeed,
    #[error("unsupported sample format `{0}`")]
    UnsupportedSampleFormat(String),
}
