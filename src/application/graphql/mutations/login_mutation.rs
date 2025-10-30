use crate::application::usecases::auth::login::LoginUseCase;
use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
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

        LoginUseCase::execute(input, &kratos_client)
            .await
            .map_err(|e| async_graphql::Error::new(e))
    }
}
