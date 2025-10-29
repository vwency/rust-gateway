use crate::application::usecases::auth::login::LoginUseCase;
use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct LoginMutation;

#[Object]
impl LoginMutation {
    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthResponse> {
        let jwt_secret = ctx.data::<String>()?;

        LoginUseCase::execute(input, jwt_secret).map_err(|e| async_graphql::Error::new(e))
    }
}
