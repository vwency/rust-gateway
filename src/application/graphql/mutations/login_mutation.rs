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
        let kratos_client = ctx.data_unchecked::<KratosClient>();

        // ✅ Правильное извлечение cookie из контекста
        let cookie = ctx
            .data_opt::<Option<String>>()
            .and_then(|opt| opt.as_ref())
            .map(|s| s.as_str());

        let (auth_response, cookies) = LoginUseCase::execute(input, kratos_client, cookie)
            .await
            .map_err(async_graphql::Error::new)?;

        // ✅ Добавляем новые cookies в ответ
        if let Some(response_cookies) = ctx.data_opt::<ResponseCookies>() {
            for cookie_str in cookies {
                response_cookies.add_cookie(cookie_str).await;
            }
        }

        Ok(auth_response)
    }
}
