use crate::application::usecases::auth::register::RegisterUseCase;
use crate::domain::auth::inputs::RegisterInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::graphql::response_cookies::ResponseCookies;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct RegisterMutation;

#[Object]
impl RegisterMutation {
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<AuthResponse> {
        let kratos_client = ctx.data_unchecked::<KratosClient>();

        // Get cookie from context
        let cookie = ctx
            .data_opt::<Option<String>>()
            .and_then(|opt| opt.as_ref())
            .map(|s| s.as_str());

        let (auth_response, cookies) = RegisterUseCase::execute(input, kratos_client, cookie)
            .await
            .map_err(async_graphql::Error::new)?;

        if let Some(response_cookies) = ctx.data_opt::<ResponseCookies>() {
            for cookie_str in cookies {
                response_cookies.add_cookie(cookie_str).await;
            }
        }

        Ok(auth_response)
    }
}
