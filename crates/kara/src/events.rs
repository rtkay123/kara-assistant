use crate::config::Configuration;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum KaraEvent {
    Close,
    ReloadConfiguration(Configuration),
    ReadingSpeech(String),
    FinalisedSpeech(String),
}
