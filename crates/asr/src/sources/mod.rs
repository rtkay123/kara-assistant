use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
        api_key: String,

        #[serde(rename = "service-url")]
        service_url: String,
    },
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
