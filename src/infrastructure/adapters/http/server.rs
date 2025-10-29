use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use std::sync::Arc;
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::application::handlers::health_check as handlers;
use crate::infrastructure::adapters::graphql::handlers::{graphql_handler, graphql_playground};
use crate::infrastructure::adapters::graphql::schema::AppSchema;

pub async fn start(schema: Arc<AppSchema>) -> std::io::Result<()> {
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
            .app_data(web::Data::from(schema.clone()))
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_handler))
                    .route(web::get().to(graphql_playground)),
            )
            .configure(handlers::configure)
    })
    .bind(("127.0.0.1", 8080))?;

    info!("✅ HTTP server successfully started on http://127.0.0.1:8080");
    info!("🚀 GraphQL Playground: http://127.0.0.1:8080/graphql");

    server.run().await
}
