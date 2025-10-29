use crate::infrastructure::adapters::graphql::schema::create_schema;
use crate::infrastructure::adapters::http::server;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

pub async fn run() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("Starting application...");

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        info!("JWT_SECRET not set, using default (change in production!)");
        "your-default-secret-key-change-in-production".to_string()
    });

    info!("Creating GraphQL schema...");
    let schema = Arc::new(create_schema(jwt_secret));

    server::start(schema).await
}
