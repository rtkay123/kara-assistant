use std::{fmt::Display, io::ErrorKind, path::PathBuf};

use res_def::dirs;
use tracing::error;

use super::Configuration;

pub fn read_config_file(file: Option<PathBuf>) -> (Configuration, Option<PathBuf>) {
    let config_file = if let Some(ref file) = file {
        Some(check_file(file, None))
    } else {
        dirs::config_dir().map(|dir| {
            let is_yaml = extension() == Config::Yaml;
            match get_config(dir.clone(), &extension().to_string()) {
                Some(res) => Some(res),
                None => {
                    if is_yaml {
                        get_config(dir, "yml")
                    } else {
                        None
                    }
                }
            }
        })
    };
    match config_file {
        Some(Some(opts)) => opts,
        #[allow(unused_variables)]
        _ => {
            #[cfg(feature = "toml")]
            let config_str = include_str!("../../../../examples/kara.toml");

            #[cfg(feature = "yaml")]
            let config_str = include_str!("../../../../examples/example.kara.yaml");

            #[cfg(feature = "json")]
            let config_str = include_str!("../../../../examples/example.kara.json");

            #[cfg(feature = "toml")]
            let config: Configuration = toml::from_str(config_str).unwrap();

            #[cfg(feature = "json")]
            let config: Configuration = serde_json::from_str(config_str).unwrap();

            #[cfg(feature = "yaml")]
            let config: Configuration = serde_yaml::from_str(config_str).unwrap();

            (config, None)
        }
    }
}

fn check_file(
    file: &PathBuf,
    backup: Option<&PathBuf>,
) -> Option<(Configuration, Option<PathBuf>)> {
    let err = |e| {
        error!("{e}");
    };

    let call_backup = |backup: Option<&PathBuf>| {
        if let Some(backup) = backup {
            check_file(backup, None)
        } else {
            None
        }
    };

    let f = std::fs::read_to_string(file);

    match f {
        #[allow(unused_variables)]
        Ok(contents) => {
            #[cfg(feature = "toml")]
            let result: Result<Configuration, _> = toml::from_str::<Configuration>(&contents);

            #[cfg(feature = "json")]
            let result: Result<Configuration, _> = serde_json::from_str::<Configuration>(&contents);

            #[cfg(feature = "yaml")]
            let result: Result<Configuration, _> = serde_yaml::from_str::<Configuration>(&contents);

            match result {
                Ok(e) => Some((e, Some(file.to_owned()))),
                Err(e) => {
                    err(format!("config: {} -> {}", file.display(), e));
                    call_backup(backup)
                }
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                err(format!("config: {} -> {}", file.display(), e));
            }
            call_backup(backup)
        }
    }
}

fn extension() -> Config {
    if cfg!(feature = "config-json") {
        Config::Json
    } else if cfg!(feature = "config-yaml") {
        Config::Yaml
    } else {
        Config::Toml
    }
}

#[derive(PartialEq, Eq)]
enum Config {
    Json,
    Toml,
    Yaml,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Config::Json => "json",
                Config::Toml => "toml",
                Config::Yaml => "yaml",
            }
        )
    }
}

fn get_config(mut dir: PathBuf, extension: &str) -> Option<(Configuration, Option<PathBuf>)> {
    let crate_name = env!("CARGO_PKG_NAME");
    let mut location = PathBuf::from(crate_name);
    let mut file = PathBuf::from(crate_name);
    file.set_extension(extension);
    location.push(file.clone());
    let mut alt = dir.clone();
    dir.push(location);
    alt.push(file);
    check_file(&dir, Some(&alt))
}
