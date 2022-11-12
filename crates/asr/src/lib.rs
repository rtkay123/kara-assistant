pub mod sources;

use crossbeam_channel::{Receiver, Sender};

pub trait RecogniseSpeech {
    type Input;

    fn transcribe(
        &self,
        audio_feed: &Receiver<Vec<Self::Input>>,
        result: &Sender<TranscriptionResult>,
    ) -> Result<()>;
}

type Result<T> = std::result::Result<T, TranscriptionError>;

pub struct TranscriptionResult {
    text: String,
    finalised: bool,
}

impl TranscriptionResult {
    fn new(text: String, finalised: bool) -> Self {
        Self { text, finalised }
    }

    pub fn transcription(&self) -> &str {
        &self.text
    }

    pub fn finalised(&self) -> bool {
        self.finalised
    }
}

#[derive(Error, Debug)]
pub enum TranscriptionError {
    #[error("An error occurred in speech recognition {0}")]
    Unknown(String),
    #[error("Could not create the model from specified path {0}")]
    LocalModel(String),
    #[error("No valid receivers")]
    SendError(String),
}

pub struct SpeechRecognisers<'a, T> {
    backends: &'a [Box<dyn RecogniseSpeech<Input = T>>],
}

impl<'a, T> SpeechRecognisers<'a, T> {
    pub fn new(backends: &'a [Box<dyn RecogniseSpeech<Input = T>>]) -> Self {
        trace!("creating speech recognition backends");
        Self { backends }
    }

    pub fn transcribe(&self, feed: &Receiver<Vec<T>>, text_sender: &Sender<TranscriptionResult>) {
        for i in self.backends {
            if let Err(e) = i.transcribe(feed, text_sender) {
                error!("{}, trying fallback", e.to_string());
            } else {
                trace!("transcription completed");
                break;
            }
        }
    }
}

pub use crossbeam_channel;
use thiserror::Error;
use tracing::{error, trace};
