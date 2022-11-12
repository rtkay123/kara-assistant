use clap::{Parser, ValueEnum};

/// A digital assistant
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// What mode to run the application in
    #[arg(short, long)]
    #[cfg(all(feature = "graphical", feature = "commandline"))]
    pub mode: Option<StartupMode>,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum StartupMode {
    /// Start a graphical session
    #[default]
    Gui,
    /// Start a commandline session
    Cli,
}
