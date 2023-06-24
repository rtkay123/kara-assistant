use tracing::info;

fn main() {
    init_logger();
}

fn init_logger() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kara=debug,other_crate=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let pkg_name = env!("CARGO_PKG_NAME");
    let pkg_ver = env!("CARGO_PKG_VERSION");
    info!(version = pkg_ver, "{} has started", pkg_name);
}
