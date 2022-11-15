mod audio;
#[cfg(feature = "commandline")]
mod cli;
mod config;
mod events;
#[cfg(feature = "graphical")]
mod graphics;

mod logger;

fn main() {
    logger::initialise_logger().unwrap();
    start();
}

#[tokio::main]
async fn start() {
    let args = config::initialise_application();
    #[cfg(all(feature = "graphical", feature = "commandline"))]
    match args.mode.unwrap_or_default() {
        config::cli::StartupMode::Gui => {
            graphics::run().await.unwrap();
        }
        config::cli::StartupMode::Cli => {
            cli::run().await.unwrap();
        }
    }

    if cfg!(feature = "graphical") {
        graphics::run().await.unwrap();
    } else if cfg!(feature = "commandline") {
        cli::run().await.unwrap();
    }
}
