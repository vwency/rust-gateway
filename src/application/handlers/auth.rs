use crate::application::usecases::auth::{
    CompleteRecovery, GetSession, InitiateRecovery, Login, Logout, Signup,
};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthDto {
    pub email: String,
    pub password: String,
    #[serde(flatten)]
    pub traits: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RecoveryDto {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct FlowIdQuery {
    pub flow: String,
}

// POST /auth/register - Регистрация нового пользователя
#[post("/auth/register")]
#[instrument]
async fn signup(data: web::Json<AuthDto>, req: HttpRequest) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let use_case = Signup::new(
        data.email.clone(),
        data.password.clone(),
        data.traits.clone(),
        cookie,
    );

    match use_case.execute().await {
        Ok(response) => {
            let mut http_response = HttpResponse::Created();

            if let Some(session_cookie) = response.session_cookie {
                http_response.append_header((
                    "Set-Cookie",
                    format!(
                        "ory_kratos_session={}; Path=/; HttpOnly; Secure; SameSite=Lax",
                        session_cookie
                    ),
                ));
            }

            http_response.json(response.identity)
        }
        Err(e) => {
            tracing::error!("Failed to complete registration: {:?}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Registration failed",
                "details": e.to_string()
            }))
        }
    }
}

// POST /auth/login - Вход в систему
#[post("/auth/login")]
#[instrument]
async fn login(data: web::Json<LoginDto>, req: HttpRequest) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let use_case = Login::new(data.identifier.clone(), data.password.clone(), cookie);

    match use_case.execute().await {
        Ok(response) => {
            let mut http_response = HttpResponse::Ok();

            if let Some(session_cookie) = response.session_cookie {
                http_response.append_header((
                    "Set-Cookie",
                    format!(
                        "ory_kratos_session={}; Path=/; HttpOnly; Secure; SameSite=Lax",
                        session_cookie
                    ),
                ));
            }

            http_response.json(response.session)
        }
        Err(e) => {
            tracing::error!("Failed to complete login: {:?}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Login failed",
                "details": e.to_string()
            }))
        }
    }
}

// POST /auth/logout - Выход из системы
#[post("/auth/logout")]
#[instrument]
async fn logout(req: HttpRequest) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if cookie.is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "No session cookie found"
        }));
    }

    let use_case = Logout::new(cookie.unwrap());

    match use_case.execute().await {
        Ok(_) => HttpResponse::Ok()
            .append_header((
                "Set-Cookie",
                "ory_kratos_session=; Path=/; Max-Age=0; HttpOnly; Secure; SameSite=Lax",
            ))
            .json(serde_json::json!({
                "message": "Logged out successfully"
            })),
        Err(e) => {
            tracing::error!("Failed to logout: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Logout failed"
            }))
        }
    }
}

// GET /auth/me - Получить информацию о текущем пользователе
#[get("/auth/me")]
#[instrument]
async fn get_current_user(req: HttpRequest) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if cookie.is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "No session cookie found"
        }));
    }

    let use_case = GetSession::new(cookie.unwrap());

    match use_case.execute().await {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => {
            tracing::error!("Failed to get session: {:?}", e);
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or expired session"
            }))
        }
    }
}

// GET /auth/recovery - Инициация восстановления пароля
#[get("/auth/recovery")]
#[instrument]
async fn init_recovery(req: HttpRequest) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let use_case = InitiateRecovery::new(cookie);

    match use_case.execute().await {
        Ok(flow) => HttpResponse::Ok().json(flow),
        Err(e) => {
            tracing::error!("Failed to initiate recovery: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to initiate recovery flow"
            }))
        }
    }
}

// POST /auth/recovery - Завершение восстановления пароля
#[post("/auth/recovery")]
#[instrument]
async fn complete_recovery(
    query: web::Query<FlowIdQuery>,
    data: web::Json<RecoveryDto>,
    req: HttpRequest,
) -> impl Responder {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let use_case = CompleteRecovery::new(query.flow.clone(), data.email.clone(), cookie);

    match use_case.execute().await {
        Ok(flow) => HttpResponse::Ok().json(flow),
        Err(e) => {
            tracing::error!("Failed to complete recovery: {:?}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Recovery failed",
                "details": e.to_string()
            }))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(signup)
        .service(login)
        .service(logout)
        .service(get_current_user)
        .service(init_recovery)
        .service(complete_recovery);
}
