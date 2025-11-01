use crate::application::usecases::auth::{GetSession, Login, Logout, Signup};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct LoginDto {
    #[serde(alias = "email")]
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
#[post("/auth/register")]
#[instrument(skip(data, req))]
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

            // Токен должен быть в response.session_token, а не в session
            if let Some(token) = &response.session_token {
                http_response.append_header((
                    "Set-Cookie",
                    format!(
                        "session_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                        token
                    ),
                ));
                tracing::info!(
                    "✅ Session token set for new user: {}",
                    response.identity.id
                );
            } else {
                tracing::warn!("⚠️ No session token returned after registration");
            }

            http_response.json(serde_json::json!({
                "identity": response.identity,
                "session": response.session,
                "message": "Registration successful"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to complete registration: {:?}", e);

            let error_string = e.to_string();
            if error_string.contains("missing field") && error_string.contains("node_type") {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Authentication service configuration error",
                    "message": "SDK version mismatch with Kratos server",
                    "solution": "Update ory_kratos_client crate to match server version",
                    "technical_details": error_string
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Registration failed",
                    "details": error_string
                }))
            }
        }
    }
}

#[post("/auth/login")]
#[instrument(skip(data, req))]
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

            if let Some(token) = response.session_token {
                http_response.append_header((
                    "Set-Cookie",
                    format!(
                        "session_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                        token
                    ),
                ));
                tracing::info!("✅ Session token set successfully");
            } else {
                tracing::warn!("⚠️ No session token received from Kratos");
            }

            http_response.json(serde_json::json!({
                "session": response.session,
                "message": "Login successful"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to complete login: {:?}", e);

            let error_string = e.to_string();
            if error_string.contains("missing field") && error_string.contains("node_type") {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Authentication service configuration error",
                    "message": "SDK version mismatch with Kratos server"
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Login failed",
                    "details": error_string
                }))
            }
        }
    }
}

#[post("/auth/logout")]
#[instrument(skip(req))]
async fn logout(req: HttpRequest) -> impl Responder {
    // Извлекаем токен из cookie
    let token = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookie_str| {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("session_token=") {
                    return Some(value.to_string());
                }
            }
            None
        });

    if token.is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "No session token found"
        }));
    }

    let use_case = Logout::new(token.unwrap());

    match use_case.execute().await {
        Ok(_) => HttpResponse::Ok()
            .append_header((
                "Set-Cookie",
                "session_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax",
            ))
            .json(serde_json::json!({
                "message": "Logged out successfully"
            })),
        Err(e) => {
            tracing::error!("Failed to logout: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Logout failed",
                "details": e.to_string()
            }))
        }
    }
}

#[get("/auth/me")]
#[instrument(skip(req))]
async fn get_current_user(req: HttpRequest) -> impl Responder {
    // Ищем токен в cookie или Authorization header
    let token = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|cookie_str| {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix("session_token=") {
                    return Some(value.to_string());
                }
            }
            None
        })
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|auth| auth.strip_prefix("Bearer "))
                .map(|t| t.to_string())
        });

    if token.is_none() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "No session token found"
        }));
    }

    let use_case = GetSession::new(token.unwrap());

    match use_case.execute().await {
        Ok(session) => HttpResponse::Ok().json(session),
        Err(e) => {
            tracing::error!("Failed to get session: {:?}", e);
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or expired session",
                "details": e.to_string()
            }))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(signup)
        .service(login)
        .service(logout)
        .service(get_current_user);
}
