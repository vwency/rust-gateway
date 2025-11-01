use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;

pub struct LoginUseCase;

impl LoginUseCase {
    pub async fn execute(
        input: LoginInput,
        kratos_client: &KratosClient,
        cookie: Option<&str>,
    ) -> Result<(AuthResponse, Vec<String>), String> {
        Self::validate_input(&input)?;

        // Проверяем, не авторизован ли пользователь уже
        if let Some(existing_cookie) = cookie {
            if let Ok(_) = kratos_client.handle_get_current_user(existing_cookie).await {
                return Err("Already logged in. Please logout first.".to_string());
            }
        }

        let identifier = input
            .email
            .as_ref()
            .or(input.username.as_ref())
            .ok_or("Email or username required")?;

        let (session, cookies) = kratos_client
            .handle_login(identifier, &input.password, cookie)
            .await
            .map_err(|e| format!("Login failed: {}", e))?;

        // Возвращаем все cookies, которые вернул Kratos
        Ok((
            AuthResponse::from_kratos_identity(session.identity, String::new()),
            cookies,
        ))
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
