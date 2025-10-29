use crate::infrastructure::adapters::graphql::schema::AppSchema;
use actix_web::{HttpResponse, Result, web};
use async_graphql::http::GraphiQLSource;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

pub async fn graphql_handler(schema: web::Data<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphql_playground() -> Result<HttpResponse> {
    let html = GraphiQLSource::build().endpoint("/graphql").finish();
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
