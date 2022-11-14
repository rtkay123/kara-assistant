pub mod kara;

use std::path::PathBuf;

use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use crate::{Transcibe, TranscriptionResult};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "source")]
pub enum Source {
    Kara {
        #[serde(rename = "model-path")]
        model_path: PathBuf,
    },

    #[serde(rename = "ibm-watson")]
    IBMWatson {
        #[serde(rename = "api-key")]
        #[serde(default = "empty_string")]
        api_key: String,

        #[serde(rename = "service-url")]
        #[serde(default = "empty_string")]
        service_url: String,
    },
}
fn empty_string() -> String {
    String::default()
}

impl Default for Source {
    fn default() -> Self {
        Self::Kara {
            model_path: PathBuf::new(),
        }
    }
}

impl ToString for Source {
    fn to_string(&self) -> String {
        match &self {
            Source::Kara { .. } => "kara",
            Source::IBMWatson { .. } => "ibm-watson",
        }
        .to_owned()
    }
}

pub struct SpeechRecognisers {
    sources: Vec<Box<dyn Transcibe>>,
}

impl SpeechRecognisers {
    pub fn new(sources: Vec<Box<dyn Transcibe>>) -> Self {
        trace!("creating speech recognition backends");
        Self { sources }
    }

    pub fn speech_to_text(&self, feed: &[i16], result_sender: &Sender<TranscriptionResult>) {
        for i in self.sources.iter() {
            if let Err(e) = i.transcribe(feed, result_sender) {
                error!("{}, trying fallback", e.to_string());
            } else {
                trace!("transcription completed");
                break;
            }
        }
    }
}
