mod config;
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
    if cfg!(all(feature = "graphical", feature = "commandline")) {
        match args.mode.unwrap_or_default() {
            config::cli::StartupMode::Gui => {
                graphics::run().await.unwrap();
            }
            config::cli::StartupMode::Cli => {
                println!("starting cli");
            }
        }
    } else if cfg!(not(feature = "graphical")) && cfg!(feature = "commandline") {
        println!("starting cli");
    } else if cfg!(not(feature = "commandline")) && cfg!(feature = "graphical") {
        #[cfg(feature = "graphical")]
        graphics::run().await.unwrap();
    }
}
