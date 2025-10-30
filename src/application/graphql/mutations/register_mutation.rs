use crate::application::usecases::auth::register::RegisterUseCase;
use crate::domain::auth::inputs::RegisterInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct RegisterMutation;

#[Object]
impl RegisterMutation {
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<AuthResponse> {
        let kratos_client = ctx.data_opt::<KratosClient>().cloned().unwrap_or_else(|| {
            KratosClient::new(
                "http://localhost:4434".to_string(),
                "http://localhost:4433".to_string(),
            )
        });

        RegisterUseCase::execute(input, &kratos_client)
            .await
            .map_err(|e| async_graphql::Error::new(e))
    }
}
