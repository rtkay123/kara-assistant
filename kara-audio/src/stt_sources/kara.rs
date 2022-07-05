use std::sync::{Arc, Mutex};

use anyhow::Context;
use tracing::trace;
use vosk::Recognizer;

use crate::SAMPLE_RATE;

use super::STTSource;

/// Initialises a Coqui STT model with an (optional) scorer (language model).
/// Panics if the STT model could not be initialised
#[tracing::instrument]
pub(crate) fn init_kara_model(model: &str) -> anyhow::Result<STTSource> {
    use gag::Gag;
    let _print_gag = Gag::stderr().unwrap();
    trace!("initialising kara stt model");
    let vosk_model = vosk::Model::new(model).context(format!(
        "failed to initialise kara stt model from path: {}",
        model
    ))?;
    let mut recogniser = Recognizer::new(&vosk_model, SAMPLE_RATE as f32)
        .context("failed to initialise recogniser")?;
    // recogniser.set_max_alternatives(10);
    recogniser.set_words(true);
    recogniser.set_partial_words(true);
    trace!(path = %model, "located model");
    trace!("kara stt model initialised");
    Ok(STTSource::Kara(Arc::new(Mutex::new(recogniser))))
}
