pub mod sources;

pub trait Transcibe: Send {
    fn source(&self) -> &str;
    fn transcribe(&self, stream: &[i16], result_sender: &Sender<TranscriptionResult>)
        -> Result<()>;
}

pub struct TranscriptionResult {
    text: String,
    finalised: bool,
}

impl TranscriptionResult {
    fn new(text: &str, finalised: bool) -> Self {
        Self {
            text: text.to_string(),
            finalised,
        }
    }

    pub fn transcription(&self) -> &str {
        &self.text
    }

    pub fn finalised(&self) -> bool {
        self.finalised
    }
}

pub use crossbeam_channel::Sender;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranscriptionError {
    #[error("An error occurred in speech recognition {0}")]
    Unknown(String),
    #[error("Could not create the model from specified path {0}")]
    LocalModel(String),
    #[error("No valid receivers")]
    SendError(String),
}

type Result<T> = std::result::Result<T, TranscriptionError>;
