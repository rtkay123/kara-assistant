mod cli;
mod debug;

fn main() {
    let (_guard, interface) = debug::initialise();
    match interface {
        cli::Interface::Cli => {
            println!("Hello, world!");
        }
        cli::Interface::Gui => {}
    }
}
