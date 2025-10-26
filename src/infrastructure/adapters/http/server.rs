use actix_cors::Cors;
use actix_web::{App, HttpServer};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::application::handlers::health_check as handlers;

pub async fn start() -> std::io::Result<()> {
    info!("Booting HTTP server at http://127.0.0.1:8080");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .configure(handlers::configure)
    })
    .bind(("127.0.0.1", 8080))?;

    info!("✅ HTTP server successfully started on http://127.0.0.1:8080");

    server.run().await
}
