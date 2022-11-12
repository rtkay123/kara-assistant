use anyhow::Result;
use tracing::info;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub(crate) fn initialise_logger() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_timer(
                tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339()
                    .expect("could not get local time offset"),
            ),
        )
        .with(
            EnvFilter::from_default_env()
                .add_directive("wgpu_core=warn".parse()?)
                .add_directive("naga=warn".parse()?)
                .add_directive("wgpu_hal=error".parse()?)
                .add_directive("iced_wgpu=warn".parse()?),
        )
        .init();

    info!(
        "{} {} has started",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}
