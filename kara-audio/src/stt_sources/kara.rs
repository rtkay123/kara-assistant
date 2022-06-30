use std::sync::Arc;

use anyhow::Context;
use tracing::{error, trace};

use super::STTSource;

/// Initialises a Coqui STT model with an (optional) scorer (language model).
/// Panics if the STT model could not be initialised
#[tracing::instrument]
pub(crate) fn init_kara_model(model: &str, scorer: &Option<String>) -> anyhow::Result<STTSource> {
    use gag::Gag;
    let _print_gag = Gag::stderr().unwrap();
    trace!("initialising kara stt model");
    let mut coqui_model = coqui_stt::Model::new(model)
        .with_context(|| format!("failed to initialise kara stt model from path: {}", model))?;
    trace!(path = %model, "located model");
    if let Some(scorer) = scorer {
        if let Err(e) = coqui_model.enable_external_scorer(scorer) {
            error!(path = %scorer, "{}", e.to_string().to_lowercase());
        } else {
            trace!(path= %scorer, "using scorer");
        }
    }
    trace!("kara stt model initialised");
    Ok(STTSource::Kara(Arc::new(coqui_model)))
}
