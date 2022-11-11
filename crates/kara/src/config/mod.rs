use clap::Parser;

use self::cli::Args;

pub mod cli;

pub fn initialise_application() -> Args {
    Args::parse()
}
