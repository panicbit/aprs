use color_eyre::eyre::{Context, Result};
use tokio::runtime::Runtime;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn default_main_setup() -> Result<Runtime> {
    color_eyre::install().unwrap();
    configure_tracing();

    let rt = Runtime::new().context("failed to create tokio runtime")?;

    Ok(rt)
}

fn configure_tracing() {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time()
                .with_writer(std::io::stderr),
        )
        .with(ErrorLayer::default())
        .init();
}
