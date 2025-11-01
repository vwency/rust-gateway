use crate::domain::auth::inputs::RegisterInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;
use validator::Validate;

#[derive(Validate)]
struct RegisterValidation {
    #[validate(email)]
    email: String,
    #[validate(length(min = 3, max = 20))]
    username: String,
    #[validate(length(min = 8))]
    password: String,
}

pub struct RegisterUseCase;

impl RegisterUseCase {
    pub async fn execute(
        input: RegisterInput,
        kratos_client: &KratosClient,
        cookie: Option<&str>,
    ) -> Result<AuthResponse, String> {
        Self::validate_input(&input)?;

        let (session, cookies) = kratos_client
            .handle_signup(&input.email, &input.username, &input.password, cookie)
            .await
            .map_err(|e| format!("Failed to register: {}", e))?;

        let session_token = cookies.get(0).cloned().unwrap_or_default();

        Ok(AuthResponse::from_kratos_identity(
            session.identity,
            session_token,
        ))
    }

    fn validate_input(input: &RegisterInput) -> Result<(), String> {
        let validation = RegisterValidation {
            email: input.email.clone(),
            username: input.username.clone(),
            password: input.password.clone(),
        };

        validation
            .validate()
            .map_err(|e| format!("Validation error: {}", e))?;
        Ok(())
    }
}
