use crate::infrastructure::adapters::http::server;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub async fn run() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("Starting application...");

    server::start().await
}
