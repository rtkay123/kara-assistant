pub mod monitor;
use std::path::Path;

use asr::sources::Source;
use clap::Parser;

use serde::{Deserialize, Deserializer, Serialize};

use self::cli::Args;

pub mod cli;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Configuration {
    #[cfg(feature = "graphical")]
    #[serde(default = "default_window")]
    pub window: Window,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Audio>,

    #[serde(rename = "speech-recognition")]
    #[serde(default = "default_recogniser")]
    pub speech_recognition: SpeechRecognition,

    #[serde(default = "colours")]
    #[cfg(feature = "graphical")]
    pub colours: Colours,
}

fn colours() -> Colours {
    Colours::default()
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Colours {
    #[serde(default = "default_background")]
    pub background: String,

    #[serde(default = "default_foreground")]
    pub foreground: String,
}

impl Default for Colours {
    fn default() -> Self {
        Self {
            background: default_background(),
            foreground: default_foreground(),
        }
    }
}

fn default_background() -> String {
    "#000000".to_owned()
}

fn default_foreground() -> String {
    "#FFFFFF".to_owned()
}

fn default_recogniser() -> SpeechRecognition {
    SpeechRecognition::default()
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Audio {
    #[serde(rename = "input-device-name")]
    pub input_device_name: Option<String>,

    #[serde(rename = "sample-rate")]
    pub sample_rate: Option<f32>,

    #[serde(default = "visualiser")]
    #[cfg(feature = "graphical")]
    pub visualiser: Visualiser,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Visualiser {
    #[serde(default = "default_loudness")]
    pub loudness: f32,

    #[serde(default = "default_density")]
    pub buffering: u16,

    #[serde(rename = "smoothing-size")]
    #[serde(default = "default_density")]
    pub smoothing_size: u16,

    #[serde(rename = "smoothing-amount")]
    #[serde(default = "default_density")]
    pub smoothing_amount: u16,

    #[serde(default = "default_resolution")]
    pub resolution: u16,

    #[serde(rename = "density-reduction")]
    #[serde(default = "default_density")]
    pub density_reduction: u16,

    #[serde(default = "vis_top")]
    #[serde(rename = "top-colour")]
    pub top_colour: String,

    #[serde(default = "vis_bottom")]
    #[serde(rename = "bottom-colour")]
    pub bottom_colour: String,

    #[serde(default = "vis_radius")]
    pub radius: f32,

    #[serde(default = "stroke")]
    pub stroke: f32,

    #[serde(default = "rotation")]
    pub rotation: f32,
}

fn stroke() -> f32 {
    1.5
}

fn vis_radius() -> f32 {
    0.25
}

fn rotation() -> f32 {
    0.0
}

impl Default for Visualiser {
    fn default() -> Self {
        Self {
            loudness: default_loudness(),
            buffering: default_density(),
            smoothing_size: default_density(),
            smoothing_amount: default_density(),
            resolution: default_resolution(),
            density_reduction: default_density(),
            top_colour: vis_top(),
            bottom_colour: vis_bottom(),
            radius: vis_radius(),
            stroke: stroke(),
            rotation: rotation(),
        }
    }
}
fn vis_top() -> String {
    String::from("#da294f")
}

fn vis_bottom() -> String {
    String::from("#02000D")
}

fn visualiser() -> Visualiser {
    Visualiser::default()
}

fn default_density() -> u16 {
    5
}
fn default_resolution() -> u16 {
    3000
}

fn default_loudness() -> f32 {
    1.5
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SpeechRecognition {
    #[serde(default = "default_source")]
    #[serde(deserialize_with = "de_source_name_only")]
    #[serde(rename = "default-source")]
    pub default_source: String,

    #[serde(default = "sources")]
    pub sources: Vec<Source>,
}

fn de_source_name_only<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Source = Deserialize::deserialize(deserializer)?;
    // do better hex decoding than this
    Ok(s.to_string())
}

impl Default for SpeechRecognition {
    fn default() -> Self {
        Self {
            default_source: Source::default().to_string(),
            sources: sources(),
        }
    }
}

fn default_source() -> String {
    Source::default().to_string()
}

fn sources() -> Vec<Source> {
    // Single source
    vec![Source::default()]
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Window {
    #[serde(default = "window_name")]
    pub title: String,

    #[serde(default = "disable_decorations")]
    pub decorations: bool,

    #[serde(default = "set_opacity")]
    pub opacity: f32,

    #[serde(default = "set_padding")]
    pub padding: u16,

    #[serde(default = "font_size")]
    #[serde(rename = "font-size")]
    pub font_size: u16,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: window_name(),
            decorations: disable_decorations(),
            opacity: set_opacity(),
            padding: set_padding(),
            font_size: font_size(),
        }
    }
}

fn set_padding() -> u16 {
    0
}

fn font_size() -> u16 {
    48
}

fn window_name() -> String {
    let crate_name = env!("CARGO_CRATE_NAME");
    format!("{}{}", &crate_name[..1], &crate_name[1..])
}

fn disable_decorations() -> bool {
    false
}

fn set_opacity() -> f32 {
    1.0
}

fn default_window() -> Window {
    Window::default()
}

pub fn read_config_file() -> Configuration {
    match dirs::config_dir() {
        Some(mut base) => {
            let mut nested = base.clone();
            nested.push("kara/kara.toml");
            if nested.exists() {
                if let Ok(Ok(file)) = read_file(&nested) {
                    file
                } else {
                    try_secondary(&mut base)
                }
            } else {
                try_secondary(&mut base)
            }
        }
        None => use_default(),
    }
}

fn read_file(
    path: impl AsRef<Path>,
) -> Result<Result<Configuration, toml::de::Error>, std::io::Error> {
    std::fs::read_to_string(&path).map(|s| toml::from_str::<Configuration>(&s))
}

fn use_default() -> Configuration {
    let bytes = include_str!("../../../../examples/kara.toml");
    match toml::from_str(bytes) {
        Ok(val) => val,
        Err(e) => panic!("{:#?}", e),
    }
}

fn try_secondary(base: &mut std::path::PathBuf) -> Configuration {
    base.push("kara.toml");
    if let Ok(Ok(file)) = read_file(&base) {
        file
    } else {
        use_default()
    }
}

pub fn initialise_application() -> Args {
    Args::parse()
}
