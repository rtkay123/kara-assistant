mod cli;
mod config;
mod debug;
mod gui;

#[tokio::main]
async fn main() {
    let (_guard, config) = debug::initialise();

    match config.general_settings.startup_mode {
        cli::Interface::Cli => {
            println!("Hello, world!");
        }
        cli::Interface::Gui => {
            if let Err(e) = gui::start(&config).await {
                tracing::error!("{}", e);
            }
        }
    }
}
