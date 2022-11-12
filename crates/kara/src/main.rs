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
            println!("starting cli");
            std::process::exit(0);
        }
    }

    #[cfg(feature = "graphical")]
    graphics::run().await.unwrap();
    #[cfg(feature = "commandline")]
    println!("starting cli");
}
