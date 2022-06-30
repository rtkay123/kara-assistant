use serde::Deserialize;

pub const DEFAULT_STT_MODEL: &str = "kara-assets/kara-stt.tflite";

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(rename = "general-settings")]
    general_settings: Option<GeneralSettings>,
    #[serde(rename = "natural-language-understanding")]
    nlu: Option<Nlu>,
}

#[derive(Debug, Deserialize)]
struct GeneralSettings {
    #[serde(rename = "default-mode")]
    default_mode: Option<String>,
    #[serde(rename = "log-level")]
    log_level: Option<String>,
    units: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Nlu {
    #[serde(rename = "speech-to-text")]
    stt: Option<SpeechToText>,
}

#[derive(Debug, Deserialize)]
struct SpeechToText {
    #[serde(rename = "pause-length")]
    pause_length: Option<f32>,
    source: Option<String>,
    kara_config: Option<STTKara>,
}

#[derive(Debug, Deserialize)]
struct STTKara {
    #[serde(rename = "model-path")]
    model_path: Option<String>,
    #[serde(rename = "scorer-path")]
    scorer_path: Option<String>,
}

pub mod state {
    use serde::Deserialize;

    use crate::cli::{DebugMode, Interface};

    use super::{ConfigFile, DEFAULT_STT_MODEL};

    #[derive(Debug, Deserialize)]
    pub enum Units {
        Metric,
        Imperial,
    }

    #[derive(Debug, Deserialize)]
    pub enum NLUSources {
        Kara(STTKara),
        Gcp,
        Watson,
    }

    #[derive(Debug, Deserialize)]
    pub struct ParsedConfig {
        #[serde(rename = "general-settings")]
        pub general_settings: GeneralSettings,
        #[serde(rename = "natural-language-understanding")]
        pub nlu: Nlu,
    }

    #[derive(Debug, Deserialize)]
    pub struct GeneralSettings {
        #[serde(rename = "default-mode")]
        pub startup_mode: Interface,
        #[serde(rename = "log-level")]
        pub log_level: DebugMode,
        pub units: Units,
    }

    #[derive(Debug, Deserialize)]
    pub struct Nlu {
        pub stt: SpeechToText,
    }

    #[derive(Debug, Deserialize)]
    pub struct SpeechToText {
        #[serde(rename = "pause-length")]
        pub pause_length: f32,
        pub source: NLUSources,
    }

    #[derive(Debug, Deserialize)]
    pub struct STTKara {
        #[serde(rename = "model-path")]
        pub model_path: String,
        #[serde(rename = "scorer-path")]
        pub scorer_path: Option<String>,
    }
    impl From<ConfigFile> for ParsedConfig {
        fn from(conf: ConfigFile) -> Self {
            let units = match &conf.general_settings {
                Some(val) => {
                    let units = match &val.units {
                        Some(units) => {
                            if units.trim().eq_ignore_ascii_case("metric") {
                                Units::Metric
                            } else if units.trim().eq_ignore_ascii_case("imperial") {
                                Units::Imperial
                            } else {
                                eprintln!("error reading units config: acceptable values are metric and imperial");
                                Units::Metric
                            }
                        }
                        None => Units::Metric,
                    };
                    units
                }
                None => Units::Metric,
            };

            let ui = match &conf.general_settings {
                Some(val) => {
                    let ui = match &val.default_mode {
                        Some(ui) => {
                            if ui.trim().eq_ignore_ascii_case("gui") {
                                Interface::Gui
                            } else if ui.trim().eq_ignore_ascii_case("cli") {
                                Interface::Cli
                            } else {
                                eprintln!(
                                    "error reading units config: acceptable values are gui and cli"
                                );
                                Interface::Gui
                            }
                        }
                        None => Interface::Gui,
                    };
                    ui
                }
                None => Interface::Gui,
            };

            let log_level = match &conf.general_settings {
                Some(val) => {
                    let log_level = match &val.log_level {
                        Some(level) => {
                            let level = level.trim().to_lowercase();
                            match level.as_str() {
                                "trace" => DebugMode::Trace,
                                "debug" => DebugMode::Debug,
                                "info" => DebugMode::Info,
                                "warn" => DebugMode::Warn,
                                "error" => DebugMode::Error,
                                _ => DebugMode::Warn,
                            }
                        }
                        None => DebugMode::Warn,
                    };
                    log_level
                }
                None => DebugMode::Warn,
            };

            let nlu = match &conf.nlu {
                Some(nlu) => match &nlu.stt {
                    Some(stt) => {
                        let pause_length = stt.pause_length.unwrap_or(2.0);
                        let source = match &stt.source {
                            Some(source) => match source.trim().to_lowercase().as_str() {
                                "kara" => {
                                    let (model_path, scorer): (String, Option<String>) =
                                        match &stt.kara_config {
                                            Some(paths) => (
                                                paths
                                                    .model_path
                                                    .as_ref()
                                                    .unwrap_or(&DEFAULT_STT_MODEL.to_owned())
                                                    .to_owned(),
                                                None,
                                            ),
                                            None => (DEFAULT_STT_MODEL.to_owned(), None),
                                        };
                                    NLUSources::Kara(STTKara {
                                        model_path,
                                        scorer_path: scorer,
                                    })
                                }
                                "watson" => {
                                    todo!()
                                }
                                _ => {
                                    todo!()
                                }
                            },
                            None => todo!(),
                        };
                        (pause_length, source)
                    }
                    None => todo!(),
                },
                None => todo!(),
            };
            Self {
                general_settings: GeneralSettings {
                    startup_mode: ui,
                    log_level,
                    units,
                },
                nlu: Nlu {
                    stt: SpeechToText {
                        pause_length: nlu.0,
                        source: nlu.1,
                    },
                },
            }
        }
    }
}
