mod debug;

fn main() {
    let _guard = debug::start_logger();
    println!("Hello, world!");
}
