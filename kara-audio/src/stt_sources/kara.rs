use std::sync::Arc;

use anyhow::Context;
use tracing::error;

use super::STTSource;

/// Initialises a Coqui STT model with an (optional) scorer (language model).
/// Panics if the STT model could not be initialised
pub(crate) fn init_kara_model(model: &str, scorer: &Option<String>) -> anyhow::Result<STTSource> {
    let mut model = coqui_stt::Model::new(model)
        .with_context(|| format!("failed to initialise kara stt model from path: {}", model))?;
    if let Some(scorer) = scorer {
        if let Err(e) = model.enable_external_scorer(scorer) {
            error!(path = %scorer, "{}", e.to_string().to_lowercase());
        }
    }
    Ok(STTSource::Kara(Arc::new(model)))
}
