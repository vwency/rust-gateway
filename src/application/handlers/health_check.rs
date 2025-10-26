use crate::application::usecases::health_check::HealthCheck;
use actix_web::{HttpResponse, Responder, get};
use tracing::instrument;

#[get("/health")]
#[instrument]
async fn health() -> impl Responder {
    let use_case = HealthCheck;
    let result = use_case.execute();
    HttpResponse::Ok().body(result)
}

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health);
}
