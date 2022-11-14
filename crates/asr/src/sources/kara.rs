use std::sync::{Arc, Mutex};

use crate::{Result, Transcibe, TranscriptionError, TranscriptionResult};
use crossbeam_channel::Sender;
use tracing::{error, trace};

pub struct LocalRecogniser {
    recogniser: Arc<Mutex<vosk::Recognizer>>,
}

impl Transcibe for LocalRecogniser {
    fn transcribe(
        &self,
        stream: &[i16],
        result_sender: &Sender<TranscriptionResult>,
    ) -> Result<()> {
        let recogniser = &mut self
            .recogniser
            .lock()
            .map_err(|f| TranscriptionError::Unknown(f.to_string()))?;
        let state = recogniser.accept_waveform(stream);
        match state {
            vosk::DecodingState::Finalized => {
                if let Some(result) = recogniser.result().single() {
                    result_sender
                        .send(TranscriptionResult {
                            text: result.text.to_string(),
                            finalised: true,
                        })
                        .map_err(|f| TranscriptionError::SendError(f.to_string()))?;
                }
            }
            vosk::DecodingState::Running => {
                result_sender
                    .send(TranscriptionResult {
                        text: recogniser.partial_result().partial.to_string(),
                        finalised: false,
                    })
                    .map_err(|f| TranscriptionError::SendError(f.to_string()))?;
            }
            vosk::DecodingState::Failed => {
                error!("local transcription failed");
            }
        }
        Ok(())
    }
}

impl LocalRecogniser {
    pub fn new(model_path: impl AsRef<std::path::Path>, sample_rate: f32) -> Result<Self> {
        trace!("using local speech recogniser");
        use gag::Gag;
        let _gag = Gag::stderr().map_err(|_| {
            TranscriptionError::Unknown(String::from("could not hijack scoped stderr output"))
        })?;
        let model_path = model_path.as_ref().to_string_lossy();

        trace!("creating local model");
        let model = vosk::Model::new(&*model_path)
            .ok_or_else(|| TranscriptionError::LocalModel(model_path.to_string()))?;

        trace!("creating local recogniser");
        let recogniser = vosk::Recognizer::new(&model, sample_rate).ok_or_else(|| {
            TranscriptionError::Unknown(String::from("Could not create recogniser from model"))
        })?;

        let recogniser = Arc::new(Mutex::new(recogniser));

        Ok(Self { recogniser })
    }
}
