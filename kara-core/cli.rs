use clap::{ArgEnum, Parser};
use serde::Deserialize;
use tracing::Level;

/// A digital assistant
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// What debug level to run the program in
    #[clap(short, long, arg_enum)]
    debug: Option<DebugMode>,
    /// What interface to use
    #[clap(short, long, arg_enum)]
    interface: Option<Interface>,
    /// Specify alternative configuration file [default: $XDG_CONFIG_HOME/kara/kara.toml]
    #[clap(short, long)]
    config: Option<String>,
}

impl Args {
    pub fn debug(&self, config_file_level: DebugMode) -> Level {
        match self.debug {
            Some(mode) => Args::map_log_level(mode),
            None => Args::map_log_level(config_file_level),
        }
    }
    fn map_log_level(mode: DebugMode) -> Level {
        match mode {
            DebugMode::Trace => Level::TRACE,
            DebugMode::Debug => Level::DEBUG,
            DebugMode::Info => Level::INFO,
            DebugMode::Warn => Level::WARN,
            DebugMode::Error => Level::ERROR,
        }
    }

    pub fn interface(&self, config_file_interface: Interface) -> Interface {
        self.interface.unwrap_or(config_file_interface)
    }

    pub fn config_path(&self) -> Option<&String> {
        self.config.as_ref()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug, Deserialize)]
pub enum DebugMode {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug, Deserialize)]
pub enum Interface {
    Cli,
    Gui,
}

impl ToString for Interface {
    fn to_string(&self) -> String {
        match self {
            Interface::Cli => "commandline",
            Interface::Gui => "graphical",
        }
        .to_owned()
    }
}

pub fn initialise() -> Args {
    Args::parse()
}
