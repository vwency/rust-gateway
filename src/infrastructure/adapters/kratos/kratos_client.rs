use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct KratosClient {
    client: Client,
    admin_url: String,
    public_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KratosIdentity {
    pub id: String,
    pub schema_id: String,
    pub traits: IdentityTraits,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityTraits {
    pub email: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KratosSession {
    pub id: String,
    pub token: String,
    pub identity_id: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
struct CreateIdentityRequest {
    schema_id: String,
    traits: IdentityTraits,
    credentials: Credentials,
}

#[derive(Debug, Serialize)]
struct Credentials {
    password: PasswordCredentials,
}

#[derive(Debug, Serialize)]
struct PasswordCredentials {
    config: PasswordConfig,
}

#[derive(Debug, Serialize)]
struct PasswordConfig {
    password: String,
}

impl KratosClient {
    pub fn new(admin_url: String, public_url: String) -> Self {
        Self {
            client: Client::new(),
            admin_url,
            public_url,
        }
    }

    pub async fn register(
        &self,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<(KratosIdentity, KratosSession), Box<dyn std::error::Error>> {
        let flow_response = self
            .client
            .get(format!("{}/self-service/registration/api", self.public_url))
            .send()
            .await?;

        if !flow_response.status().is_success() {
            let error_text = flow_response.text().await?;
            return Err(format!("Failed to initialize registration flow: {}", error_text).into());
        }

        let flow: serde_json::Value = flow_response.json().await?;
        let flow_id = flow["id"].as_str().ok_or("Flow ID not found")?.to_string();

        let registration_request = serde_json::json!({
            "method": "password",
            "traits": {
                "email": email,
                "username": username
            },
            "password": password,
        });

        let response = self
            .client
            .post(format!(
                "{}/self-service/registration?flow={}",
                self.public_url, flow_id
            ))
            .json(&registration_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Registration failed: {}", error_text).into());
        }

        let response_data: serde_json::Value = response.json().await?;

        let session = KratosSession {
            id: response_data["session"]["id"]
                .as_str()
                .ok_or("Session ID not found")?
                .to_string(),
            token: response_data["session_token"]
                .as_str()
                .ok_or("Session token not found")?
                .to_string(),
            identity_id: response_data["session"]["identity"]["id"]
                .as_str()
                .ok_or("Identity ID not found")?
                .to_string(),
            active: response_data["session"]["active"]
                .as_bool()
                .unwrap_or(false),
        };

        let identity = KratosIdentity {
            id: response_data["identity"]["id"]
                .as_str()
                .ok_or("Identity ID not found")?
                .to_string(),
            schema_id: response_data["identity"]["schema_id"]
                .as_str()
                .unwrap_or("default")
                .to_string(),
            traits: IdentityTraits {
                email: response_data["identity"]["traits"]["email"]
                    .as_str()
                    .ok_or("Email not found")?
                    .to_string(),
                username: response_data["identity"]["traits"]["username"]
                    .as_str()
                    .ok_or("Username not found")?
                    .to_string(),
            },
            created_at: response_data["identity"]["created_at"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            updated_at: response_data["identity"]["updated_at"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        };

        Ok((identity, session))
    }

    #[allow(unused)]
    pub async fn create_identity(
        &self,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<KratosIdentity, Box<dyn std::error::Error>> {
        let request = CreateIdentityRequest {
            schema_id: "default".to_string(),
            traits: IdentityTraits {
                email: email.to_string(),
                username: username.to_string(),
            },
            credentials: Credentials {
                password: PasswordCredentials {
                    config: PasswordConfig {
                        password: password.to_string(),
                    },
                },
            },
        };

        let response = self
            .client
            .post(format!("{}/admin/identities", self.admin_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Kratos error: {}", error_text).into());
        }

        let identity: KratosIdentity = response.json().await?;
        Ok(identity)
    }

    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<KratosSession, Box<dyn std::error::Error>> {
        // Initialize login flow
        let flow_response = self
            .client
            .get(format!("{}/self-service/login/api", self.public_url))
            .send()
            .await?;

        let flow: serde_json::Value = flow_response.json().await?;
        let flow_id = flow["id"].as_str().ok_or("Flow ID not found")?.to_string();

        let login_request = serde_json::json!({
            "method": "password",
            "identifier": identifier,
            "password": password,
        });

        let response = self
            .client
            .post(format!(
                "{}/self-service/login?flow={}",
                self.public_url, flow_id
            ))
            .json(&login_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Login failed: {}", error_text).into());
        }

        let session_data: serde_json::Value = response.json().await?;

        let session = KratosSession {
            id: session_data["session"]["id"]
                .as_str()
                .ok_or("Session ID not found")?
                .to_string(),
            token: session_data["session_token"]
                .as_str()
                .ok_or("Session token not found")?
                .to_string(),
            identity_id: session_data["session"]["identity"]["id"]
                .as_str()
                .ok_or("Identity ID not found")?
                .to_string(),
            active: session_data["session"]["active"].as_bool().unwrap_or(false),
        };

        Ok(session)
    }

    pub async fn get_identity(
        &self,
        identity_id: &str,
    ) -> Result<KratosIdentity, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!(
                "{}/admin/identities/{}",
                self.admin_url, identity_id
            ))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to get identity: {}", error_text).into());
        }

        let identity: KratosIdentity = response.json().await?;
        Ok(identity)
    }

    #[allow(unused)]
    pub async fn validate_session(
        &self,
        session_token: &str,
    ) -> Result<KratosSession, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!("{}/sessions/whoami", self.public_url))
            .header("Authorization", format!("Bearer {}", session_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Invalid session".into());
        }

        let session_data: serde_json::Value = response.json().await?;

        let session = KratosSession {
            id: session_data["id"]
                .as_str()
                .ok_or("Session ID not found")?
                .to_string(),
            token: session_token.to_string(),
            identity_id: session_data["identity"]["id"]
                .as_str()
                .ok_or("Identity ID not found")?
                .to_string(),
            active: session_data["active"].as_bool().unwrap_or(false),
        };

        Ok(session)
    }

    #[allow(unused)]
    pub async fn logout(&self, session_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self
            .client
            .delete(format!("{}/sessions", self.public_url))
            .header("Authorization", format!("Bearer {}", session_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Logout failed: {}", error_text).into());
        }

        Ok(())
    }
}
