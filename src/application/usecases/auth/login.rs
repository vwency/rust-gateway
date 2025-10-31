use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;

pub struct LoginUseCase;

impl LoginUseCase {
    pub async fn execute(
        input: LoginInput,
        kratos_client: &KratosClient,
    ) -> Result<(AuthResponse, String), String> {
        Self::validate_input(&input)?;

        // Определяем идентификатор (email или username)
        let identifier = input
            .email
            .as_ref()
            .or(input.username.as_ref())
            .ok_or("Email or username required")?;

        // Аутентификация через Kratos
        let session = kratos_client
            .login(identifier, &input.password)
            .await
            .map_err(|e| format!("Login failed: {}", e))?;

        // Получаем информацию об identity
        let identity = kratos_client
            .get_identity(&session.identity_id)
            .await
            .map_err(|e| format!("Failed to get identity: {}", e))?;

        let session_token = session.token.clone();
        Ok((AuthResponse::from_kratos_identity(identity), session_token))
    }

    fn validate_input(input: &LoginInput) -> Result<(), String> {
        if input.email.is_none() && input.username.is_none() {
            return Err("Email or username required".to_string());
        }

        if input.password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        if let Some(ref email) = input.email {
            if email.is_empty() {
                return Err("Email cannot be empty".to_string());
            }
        }

        if let Some(ref username) = input.username {
            if username.is_empty() {
                return Err("Username cannot be empty".to_string());
            }
        }

        Ok(())
    }
}
