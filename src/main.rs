use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use tracing::{info, instrument};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{EnvFilter, fmt};

#[get("/health")]
#[instrument]
async fn health() -> impl Responder {
    info!("Health check requested");
    HttpResponse::Ok().body("OK")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting server at http://127.0.0.1:8080");

    let cors = Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header();

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .service(health)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
