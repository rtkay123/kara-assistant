use crate::config::Configuration;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum KaraEvent {
    Close,
    ReloadConfiguration(Box<Configuration>),
    ReadingSpeech(String),
    FinalisedSpeech(String),
    UpdateProgressBar(f32),
}
