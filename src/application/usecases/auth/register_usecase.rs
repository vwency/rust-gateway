use crate::application::usecases::auth::jwt_service::JwtService;
use crate::domain::auth::inputs::RegisterInput;
use crate::domain::auth::responses::AuthResponse;
use crate::domain::entities::user::{User, UserView};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

#[derive(Validate)]
struct RegisterValidation {
    #[validate(email)]
    email: String,
    #[validate(length(min = 3, max = 20))]
    login: String,
    #[validate(length(min = 8))]
    password: String,
}

pub struct RegisterUseCase;

impl RegisterUseCase {
    pub fn execute(input: RegisterInput, jwt_secret: &str) -> Result<AuthResponse, String> {
        let validation = RegisterValidation {
            email: input.email.clone(),
            login: input.login.clone(),
            password: input.password.clone(),
        };

        validation
            .validate()
            .map_err(|e| format!("Validation error: {}", e))?;

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let user = User {
            id: user_id.clone(),
            email: input.email.clone(),
            login: input.login,
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
