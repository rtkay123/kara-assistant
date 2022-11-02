mod graphics;
mod logger;

fn main() {
    logger::initialise_logger().unwrap();
    start();
}

#[tokio::main]
async fn start() {
    graphics::run().await.unwrap();
}
