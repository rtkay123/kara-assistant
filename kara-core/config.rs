use serde::Deserialize;

pub const DEFAULT_STT_MODEL: &str = "kara-assets/stt";

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(rename = "general-settings")]
    general_settings: Option<GeneralSettings>,
    #[serde(rename = "natural-language-understanding")]
    nlu: Option<Nlu>,
    window: Option<Window>,
}

#[derive(Debug, Deserialize)]
struct Window {
    opacity: Option<f32>,
    decorations: Option<bool>,
    title: Option<String>,
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
    #[serde(rename = "kara")]
    kara_config: Option<STTKara>,
}

#[derive(Debug, Deserialize)]
struct STTKara {
    #[serde(rename = "model-path")]
    model_path: Option<String>,
}

pub mod state {
    use kara_audio::stt_sources::STTConfig;
    use serde::Deserialize;

    use crate::cli::{DebugMode, Interface};

    use super::{ConfigFile, DEFAULT_STT_MODEL};

    #[derive(Debug, Deserialize)]
    pub enum Units {
        Metric,
        Imperial,
    }
    #[derive(Debug, Deserialize)]
    pub struct ParsedConfig {
        #[serde(rename = "general-settings")]
        pub general_settings: GeneralSettings,
        #[serde(rename = "natural-language-understanding")]
        pub nlu: Nlu,
        pub window: Window,
    }

    #[derive(Debug, Deserialize)]
    pub struct Window {
        pub opacity: f32,
        pub decorations: bool,
        pub title: String,
    }

    impl Default for Window {
        fn default() -> Self {
            Self {
                opacity: 1.0,
                decorations: Default::default(),
                title: Window::get_app_name(),
            }
        }
    }

    impl Window {
        fn get_app_name() -> String {
            let title = env!("CARGO_BIN_NAME");
            format!("{}{}", &title[0..1].to_uppercase(), &title[1..])
        }
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
        pub source: STTConfig,
    }

    impl Default for SpeechToText {
        fn default() -> Self {
            Self {
                pause_length: 1.5,
                source: STTConfig::base(DEFAULT_STT_MODEL),
            }
        }
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
                                    let model_path: String = match &stt.kara_config {
                                        Some(paths) => {
                                            let mp = paths.model_path.as_ref();
                                            let mp = match mp {
                                                Some(mp) => match mp.is_empty() {
                                                    true => DEFAULT_STT_MODEL,
                                                    false => mp,
                                                },
                                                None => DEFAULT_STT_MODEL,
                                            };
                                            mp.to_owned()
                                        }
                                        None => DEFAULT_STT_MODEL.to_owned(),
                                    };
                                    STTConfig::Kara(model_path)
                                }
                                "watson" => {
                                    todo!()
                                }
                                _ => {
                                    todo!()
                                }
                            },
                            None => STTConfig::Kara(DEFAULT_STT_MODEL.to_owned()),
                        };
                        (pause_length, source)
                    }
                    None => (1.5, STTConfig::Kara(DEFAULT_STT_MODEL.to_owned())),
                },
                None => (1.5, STTConfig::Kara(DEFAULT_STT_MODEL.to_owned())),
            };
            let window = match &conf.window {
                Some(win) => {
                    let title = win
                        .title
                        .as_ref()
                        .to_owned()
                        .cloned()
                        .unwrap_or_else(Window::get_app_name);
                    let decorations = win.decorations.unwrap_or_default();
                    let opacity = win.opacity.unwrap_or(1.0);
                    Window {
                        title,
                        decorations,
                        opacity,
                    }
                }
                None => Window::default(),
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
                window,
            }
        }
    }
}
