use crate::application::usecases::auth::jwt_service::JwtService;
use crate::domain::auth::inputs::LoginInput;
use crate::domain::auth::responses::AuthResponse;
use crate::domain::entities::user::User;
use chrono::Utc;
use uuid::Uuid;

pub struct LoginUseCase;

impl LoginUseCase {
    pub fn execute(input: LoginInput, jwt_secret: &str) -> Result<AuthResponse, String> {
        if (input.email.is_none() && input.username.is_none()) || input.password.is_empty() {
            return Err("Invalid credentials".to_string());
        }

        // Проверяем конкретные значения
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
        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let user = User {
            id: user_id.clone(),
            email: input
                .email
                .unwrap_or_else(|| "placeholder@example.com".to_string()),
            login: input.username.unwrap_or_else(|| "placeholder".to_string()),
            password_hash: String::new(),
            created_at: now,
            updated_at: now,
        };

        let token = JwtService::generate_token(&user.id, &user.email, jwt_secret)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }
}
