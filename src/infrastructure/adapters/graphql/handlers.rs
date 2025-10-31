use crate::infrastructure::adapters::graphql::response_cookies::ResponseCookies;
use crate::infrastructure::adapters::graphql::schema::AppSchema;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use async_graphql::http::GraphiQLSource;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};

pub async fn graphql_handler(
    schema: web::Data<AppSchema>,
    req: GraphQLRequest,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let response_cookies = ResponseCookies::new();

    let mut request = req.into_inner();
    request = request.data(response_cookies.clone());

    let response = schema.execute(request).await;

    let cookies = response_cookies.get_cookies().await;

    let mut http_response = HttpResponse::Ok();

    for cookie in cookies {
        http_response.insert_header(("Set-Cookie", cookie));
    }

    Ok(http_response.json(response))
}

pub async fn graphql_playground() -> Result<HttpResponse> {
    let html = GraphiQLSource::build().endpoint("/graphql").finish();
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
