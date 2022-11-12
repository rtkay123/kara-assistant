use std::sync::{Arc, Mutex};

use tracing::{error, info, trace};

use crate::{RecogniseSpeech, Result, TranscriptionError, TranscriptionResult};

pub struct LocalRecogniser {
    recogniser: Arc<Mutex<vosk::Recognizer>>,
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

impl RecogniseSpeech for LocalRecogniser {
    type Input = i16;

    fn transcribe(
        &self,
        audio_feed: &crossbeam_channel::Receiver<Vec<Self::Input>>,
        text_result: &crossbeam_channel::Sender<TranscriptionResult>,
    ) -> Result<()> {
        let recogniser = &mut self
            .recogniser
            .lock()
            .map_err(|f| TranscriptionError::Unknown(f.to_string()))?;
        while let Ok(ref buffer) = audio_feed.recv() {
            let state = recogniser.accept_waveform(buffer);
            match state {
                vosk::DecodingState::Finalized => {
                    if let Some(result) = recogniser.result().single() {
                        info!("RESULT: {}", result.text);
                        text_result
                            .send(TranscriptionResult::new(result.text.to_string(), true))
                            .map_err(|f| TranscriptionError::SendError(f.to_string()))?;
                    }
                }
                vosk::DecodingState::Running => {
                    trace!("LIVE: {}", recogniser.partial_result().partial);
                    text_result
                        .send(TranscriptionResult::new(
                            recogniser.partial_result().partial.to_string(),
                            false,
                        ))
                        .map_err(|f| TranscriptionError::SendError(f.to_string()))?;
                }
                vosk::DecodingState::Failed => {
                    error!("local transcription failed");
                }
            }
        }
        Ok(())
    }
}
