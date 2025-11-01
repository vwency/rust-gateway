use actix_cors::Cors;
use actix_web::{App, HttpServer};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::application::handlers::{auth, health_check};

pub async fn start() -> std::io::Result<()> {
    info!("Booting HTTP server at http://127.0.0.1:8080");

    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials(),
            )
            .configure(health_check::configure)
            .configure(auth::configure)
    })
    .bind(("127.0.0.1", 8080))?;

    info!("✅ HTTP server successfully started on http://127.0.0.1:8080");

    server.run().await
}
