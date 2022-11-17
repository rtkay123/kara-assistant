use crate::config::Configuration;

#[derive(Debug, Clone)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum KaraEvent {
    Close,
    ReloadConfiguration(Box<Configuration>),
    ReadingSpeech(String),
    FinalisedSpeech(String),
    UpdateProgressBar(f32),
}
