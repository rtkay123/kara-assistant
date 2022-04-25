use clap::{ArgEnum, Parser};
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
}

impl Args {
    pub fn debug(&self) -> Level {
        match self.debug {
            Some(mode) => match mode {
                DebugMode::Trace => Level::TRACE,
                DebugMode::Debug => Level::DEBUG,
                DebugMode::Info => Level::INFO,
                DebugMode::Warn => Level::WARN,
                DebugMode::Error => Level::ERROR,
            },
            None => Level::WARN,
        }
    }

    pub fn interface(&self) -> Interface {
        self.interface.unwrap_or(Interface::Gui)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum DebugMode {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
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
