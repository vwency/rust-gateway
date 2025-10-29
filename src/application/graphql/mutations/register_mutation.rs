use crate::application::usecases::auth::register::RegisterUseCase;
use crate::domain::auth::inputs::RegisterInput;
use crate::domain::auth::responses::AuthResponse;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct RegisterMutation;

#[Object]
impl RegisterMutation {
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<AuthResponse> {
        let jwt_secret = ctx.data::<String>()?;

        RegisterUseCase::execute(input, jwt_secret).map_err(|e| async_graphql::Error::new(e))
    }
}
