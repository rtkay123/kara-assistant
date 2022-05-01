mod cli;
mod debug;
mod wgpu;

#[tokio::main]
async fn main() {
    let (_guard, interface) = debug::initialise();
    match interface {
        cli::Interface::Cli => {
            println!("Hello, world!");
        }
        cli::Interface::Gui => {
            wgpu::start().await.unwrap();
        }
    }
}
