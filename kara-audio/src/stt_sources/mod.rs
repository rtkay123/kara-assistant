use std::sync::Arc;

use serde::Deserialize;

use self::kara::init_kara_model;

pub mod kara;

/// Store the configurations/credentials for all the services that
/// provide STT
#[derive(Debug, Deserialize)]
pub enum STTConfig {
    Kara(String, Option<String>),
    Gcp,
    Watson,
}

impl STTConfig {
    pub fn base(path: &str) -> Self {
        STTConfig::Kara(path.to_owned(), None)
    }
}

// Store coqui on all variants as fallback?
#[derive(Clone)]
pub enum STTSource {
    Kara(Arc<coqui_stt::Model>),
    Gcp,
    Watson,
}

#[tracing::instrument]
pub fn stt_source(source: &STTConfig) -> anyhow::Result<STTSource> {
    match source {
        STTConfig::Kara(model, scorer) => init_kara_model(model, scorer),
        STTConfig::Gcp => todo!(),
        STTConfig::Watson => todo!(),
    }
}
