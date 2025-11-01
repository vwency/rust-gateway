use crate::application::usecases::auth::login::LoginUseCase;
use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::graphql::response_cookies::ResponseCookies;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct LoginMutation;

#[Object]
impl LoginMutation {
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthResponse> {
        let kratos_client = ctx.data_opt::<KratosClient>().cloned().unwrap_or_else(|| {
            KratosClient::new(
                "http://localhost:4434".to_string(),
                "http://localhost:4433".to_string(),
            )
        });

        let cookie = ctx.data_opt::<String>().map(|s| s.as_str());

        let (auth_response, cookies) = LoginUseCase::execute(input, &kratos_client, cookie)
            .await
            .map_err(|e| async_graphql::Error::new(e))?;

        if let Some(response_cookies) = ctx.data_opt::<ResponseCookies>() {
            for cookie_str in cookies {
                response_cookies.add_cookie(cookie_str).await;
            }
        }

        Ok(auth_response)
    }
}
