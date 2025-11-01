use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
use crate::infrastructure::adapters::kratos::kratos_client::KratosClient;
use tracing::{debug, error, info};

pub struct LoginUseCase;

impl LoginUseCase {
    pub async fn execute(
        input: LoginInput,
        kratos_client: &KratosClient,
        cookie: Option<&str>,
    ) -> Result<(AuthResponse, Vec<String>), String> {
        Self::validate_input(&input)?;

        let identifier = input
            .email
            .as_ref()
            .or(input.username.as_ref())
            .ok_or("Email or username required")?;

        info!(
            identifier = identifier,
            cookie_present = cookie.is_some(),
            "Starting login process"
        );

        // ✅ Проверяем наличие активной сессии и ВОЗВРАЩАЕМ ОШИБКУ
        if let Some(cookie) = cookie {
            if let Ok(Some(_session)) = kratos_client.get_session(cookie).await {
                error!("Login attempt with active session for {}", identifier);
                return Err(
                    "Already logged in. Please logout first before logging in again.".to_string(),
                );
            }
        }

        // ✅ Если сессии нет — выполняем логин
        let (session, cookies) = match kratos_client
            .handle_login(identifier, &input.password, None) // ⚠️ Передаём None, чтобы не путать cookies
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let error_msg = e.to_string();
                error!(error = %error_msg, "Login failed");
                return Err(format!("Login failed: {}", error_msg));
            }
        };

        if cookies.is_empty() {
            debug!("No cookies returned from Kratos");
        } else {
            debug!(
                cookies_count = cookies.len(),
                cookies = ?cookies,
                "Cookies returned from Kratos"
            );
        }

        info!("Login successful for identifier={}", identifier);

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
