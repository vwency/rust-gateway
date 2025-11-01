use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    NetworkError(String),
    KratosError(String),
    ValidationError(String),
    SerializationError(String),
    Unauthorized,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AuthError::KratosError(msg) => write!(f, "Kratos error: {}", msg),
            AuthError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AuthError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            AuthError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl Error for AuthError {}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        AuthError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for AuthError {
    fn from(err: serde_json::Error) -> Self {
        AuthError::SerializationError(err.to_string())
    }
}

// ============================================================================
// Domain Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: String,
    pub schema_id: String,
    pub traits: Value,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub active: bool,
    pub identity: Identity,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub authenticated_at: Option<String>,
}

#[derive(Debug)]
pub struct SignupResponse {
    pub identity: Identity,
    pub session: Option<Session>,
    pub session_token: Option<String>,
}

#[derive(Debug)]
pub struct LoginResponse {
    pub session: Session,
    pub session_token: Option<String>,
}

// ============================================================================
// Kratos Client
// ============================================================================

#[derive(Clone)]
struct KratosClient {
    client: Client,
    base_url: String,
}

impl KratosClient {
    fn new() -> Self {
        let base_url = std::env::var("KRATOS_PUBLIC_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:4433".to_string());

        Self {
            client: Client::new(),
            base_url,
        }
    }

    async fn create_registration_flow(&self, cookie: Option<String>) -> Result<Value, AuthError> {
        let url = format!("{}/self-service/registration/api", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(cookie) = cookie {
            request = request.header("Cookie", cookie);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::KratosError(format!(
                "Failed to create registration flow: {} - {}",
                status, text
            )));
        }

        let flow: Value = response.json().await?;
        Ok(flow)
    }

    async fn submit_registration(
        &self,
        flow_id: &str,
        email: String,
        password: String,
        traits: Value,
        cookie: Option<String>,
    ) -> Result<(Identity, Option<Session>, Option<String>), AuthError> {
        let url = format!(
            "{}/self-service/registration?flow={}",
            self.base_url, flow_id
        );

        let mut traits_obj = serde_json::json!({
            "email": email
        });

        if let Value::Object(ref mut map) = traits_obj {
            if let Value::Object(additional) = traits {
                map.extend(additional);
            }
        }

        let body = serde_json::json!({
            "method": "password",
            "password": password,
            "traits": traits_obj
        });

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body);

        if let Some(cookie) = cookie {
            request = request.header("Cookie", cookie);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::KratosError(format!(
                "Registration failed: {} - {}",
                status, text
            )));
        }

        let response_body: Value = response.json().await?;

        tracing::debug!(
            "Registration response: {}",
            serde_json::to_string_pretty(&response_body).unwrap_or_default()
        );

        // Извлекаем session_token с верхнего уровня
        let session_token = response_body
            .get("session_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Проверяем наличие session
        if let Some(session_data) = response_body.get("session") {
            let session: Session = serde_json::from_value(session_data.clone())?;
            tracing::info!("Session created with token: {}", session_token.is_some());
            Ok((session.identity.clone(), Some(session), session_token))
        } else if let Some(identity_data) = response_body.get("identity") {
            let identity: Identity = serde_json::from_value(identity_data.clone())?;
            tracing::warn!("No session in response, only identity");
            Ok((identity, None, session_token))
        } else {
            Err(AuthError::KratosError(
                "Unexpected response format: no session or identity found".to_string(),
            ))
        }
    }

    async fn create_login_flow(&self, cookie: Option<String>) -> Result<Value, AuthError> {
        let url = format!("{}/self-service/login/api", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(cookie) = cookie {
            request = request.header("Cookie", cookie);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::KratosError(format!(
                "Failed to create login flow: {} - {}",
                status, text
            )));
        }

        let flow: Value = response.json().await?;
        Ok(flow)
    }

    async fn submit_login(
        &self,
        flow_id: &str,
        identifier: String,
        password: String,
        cookie: Option<String>,
    ) -> Result<(Session, Option<String>), AuthError> {
        let url = format!("{}/self-service/login?flow={}", self.base_url, flow_id);

        let body = serde_json::json!({
            "method": "password",
            "identifier": identifier,
            "password": password
        });

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body);

        if let Some(cookie) = cookie {
            request = request.header("Cookie", cookie);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AuthError::KratosError(format!(
                "Login failed: {} - {}",
                status, text
            )));
        }

        let response_body: Value = response.json().await?;

        tracing::debug!(
            "Login response: {}",
            serde_json::to_string_pretty(&response_body).unwrap_or_default()
        );

        // КЛЮЧЕВОЕ ИСПРАВЛЕНИЕ: session_token на верхнем уровне ответа
        let session_token = response_body
            .get("session_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let session: Session = if let Some(session_data) = response_body.get("session") {
            serde_json::from_value(session_data.clone())?
        } else {
            serde_json::from_value(response_body)?
        };

        tracing::info!(
            "Login successful, token present: {}",
            session_token.is_some()
        );

        Ok((session, session_token))
    }

    async fn logout_session(&self, token: String) -> Result<(), AuthError> {
        let url = format!("{}/self-service/logout/api", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Session-Token", token.clone())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::KratosError(
                "Failed to get logout token".to_string(),
            ));
        }

        let logout_data: Value = response.json().await?;

        if let Some(logout_token) = logout_data.get("logout_token").and_then(|v| v.as_str()) {
            let logout_url = format!(
                "{}/self-service/logout?token={}",
                self.base_url, logout_token
            );

            let logout_response = self
                .client
                .get(&logout_url)
                .header("X-Session-Token", token)
                .send()
                .await?;

            if !logout_response.status().is_success() {
                return Err(AuthError::KratosError("Logout failed".to_string()));
            }
        }

        Ok(())
    }

    async fn get_session(&self, token: String) -> Result<Session, AuthError> {
        let url = format!("{}/sessions/whoami", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-Session-Token", token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AuthError::Unauthorized);
        }

        let session: Session = response.json().await?;
        Ok(session)
    }
}

// ============================================================================
// Use Cases
// ============================================================================

pub struct Signup {
    email: String,
    password: String,
    traits: Value,
    cookie: Option<String>,
}

impl Signup {
    pub fn new(email: String, password: String, traits: Value, cookie: Option<String>) -> Self {
        Self {
            email,
            password,
            traits,
            cookie,
        }
    }

    pub async fn execute(self) -> Result<SignupResponse, AuthError> {
        if self.email.is_empty() {
            return Err(AuthError::ValidationError("Email is required".to_string()));
        }

        if self.password.len() < 8 {
            return Err(AuthError::ValidationError(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        let client = KratosClient::new();
        let flow = client.create_registration_flow(self.cookie.clone()).await?;

        let flow_id = flow
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::KratosError("Missing flow id".to_string()))?;

        let (identity, session, session_token) = client
            .submit_registration(flow_id, self.email, self.password, self.traits, self.cookie)
            .await?;

        Ok(SignupResponse {
            identity,
            session,
            session_token,
        })
    }
}

pub struct Login {
    identifier: String,
    password: String,
    cookie: Option<String>,
}

impl Login {
    pub fn new(identifier: String, password: String, cookie: Option<String>) -> Self {
        Self {
            identifier,
            password,
            cookie,
        }
    }

    pub async fn execute(self) -> Result<LoginResponse, AuthError> {
        if self.identifier.is_empty() {
            return Err(AuthError::ValidationError(
                "Identifier is required".to_string(),
            ));
        }

        if self.password.is_empty() {
            return Err(AuthError::ValidationError(
                "Password is required".to_string(),
            ));
        }

        let client = KratosClient::new();
        let flow = client.create_login_flow(self.cookie.clone()).await?;

        let flow_id = flow
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::KratosError("Missing flow id".to_string()))?;

        let (session, session_token) = client
            .submit_login(flow_id, self.identifier, self.password, self.cookie)
            .await?;

        Ok(LoginResponse {
            session,
            session_token,
        })
    }
}

pub struct Logout {
    token: String,
}

impl Logout {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub async fn execute(self) -> Result<(), AuthError> {
        if self.token.is_empty() {
            return Err(AuthError::ValidationError(
                "Session token is required".to_string(),
            ));
        }

        let client = KratosClient::new();
        client.logout_session(self.token).await?;

        Ok(())
    }
}

pub struct GetSession {
    token: String,
}

impl GetSession {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub async fn execute(self) -> Result<Session, AuthError> {
        if self.token.is_empty() {
            return Err(AuthError::ValidationError(
                "Session token is required".to_string(),
            ));
        }

        let client = KratosClient::new();
        let session = client.get_session(self.token).await?;

        Ok(session)
    }
}
