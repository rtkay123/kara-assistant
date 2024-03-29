pub mod kara;

use std::{collections::VecDeque, path::PathBuf};

use crossbeam_channel::Sender;

use res_def::{model_path, vosk_model_url};
use serde::{Deserialize, Serialize};
use tracing::{error, info, trace};

use crate::{Transcibe, TranscriptionError, TranscriptionResult};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "source")]
pub enum Source {
    Kara {
        #[serde(rename = "model-path")]
        #[serde(default = "model_path")]
        model_path: PathBuf,

        #[serde(rename = "fallback-url")]
        #[serde(default = "vosk_link")]
        fallback_url: String,
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

fn vosk_link() -> String {
    vosk_model_url()
}

fn empty_string() -> String {
    String::default()
}

impl Default for Source {
    fn default() -> Self {
        Self::Kara {
            model_path: PathBuf::new(),
            fallback_url: vosk_link(),
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

#[derive(Default)]
pub struct SpeechRecognisers {
    sources: VecDeque<Box<dyn Transcibe>>,
}

impl SpeechRecognisers {
    pub fn new() -> Self {
        trace!("creating speech recognition backends");
        Self::default()
    }

    pub fn add(&mut self, source: Box<dyn Transcibe>) {
        let source_name = source.source();
        trace!(source = source_name, "adding speech recognition backend");
        self.sources.push_back(source);
    }

    pub fn add_primary(&mut self, source: Box<dyn Transcibe>) {
        let source_name = source.source().to_string();
        trace!(source = source_name, "setting primary backend");
        self.sources.push_front(source);
        info!(source = source_name, "using primary backend");
    }

    pub fn valid(&self) -> bool {
        !self.sources.is_empty()
    }

    pub fn speech_to_text(
        &self,
        feed: &[i16],
        result_sender: &Sender<TranscriptionResult>,
    ) -> Result<(), TranscriptionError> {
        for i in self.sources.iter() {
            if let Err(e) = i.transcribe(feed, result_sender) {
                error!(source = i.source(), "{}, trying fallback", e.to_string());
            } else {
                // trace!("transcription completed");
                break;
            }
        }
        Ok(())
    }
}
