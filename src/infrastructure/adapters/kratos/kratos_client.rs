use reqwest::{Client, header};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KratosSession {
    pub id: String,
    pub token: String,
    pub identity_id: String,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct FlowResult {
    pub flow: serde_json::Value,
    pub csrf_token: String,
    pub cookies: String,
}

#[derive(Debug, Clone)]
pub struct PostFlowResult {
    pub data: serde_json::Value,
    pub cookies: Option<String>,
}

impl KratosClient {
    pub fn new(admin_url: String, public_url: String) -> Self {
        let client = Client::builder()
            .cookie_store(false)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            admin_url,
            public_url,
        }
    }

    async fn fetch_flow(
        &self,
        flow_type: &str,
        cookie: Option<&str>,
    ) -> Result<FlowResult, Box<dyn std::error::Error>> {
        let mut request = self.client.get(format!(
            "{}/self-service/{}/api",
            self.public_url, flow_type
        ));

        if let Some(cookie_value) = cookie {
            request = request.header(header::COOKIE, cookie_value);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Failed to initialize {} flow: {}", flow_type, error_text).into());
        }

        let cookies = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("; ");

        let flow: serde_json::Value = response.json().await?;

        let csrf_token = flow["ui"]["nodes"]
            .as_array()
            .and_then(|nodes| {
                nodes
                    .iter()
                    .find(|node| node["attributes"]["name"].as_str() == Some("csrf_token"))
            })
            .and_then(|node| node["attributes"]["value"].as_str())
            .ok_or("CSRF token not found")?
            .to_string();

        Ok(FlowResult {
            flow,
            csrf_token,
            cookies,
        })
    }

    async fn post_flow(
        &self,
        endpoint: &str,
        flow_id: &str,
        data: serde_json::Value,
        cookies: &str,
    ) -> Result<PostFlowResult, Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(format!(
                "{}/self-service/{}?flow={}",
                self.public_url, endpoint, flow_id
            ))
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookies)
            .json(&data)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("{} failed: {}", endpoint, error_text).into());
        }

        let response_cookies = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("; ");

        let data: serde_json::Value = response.json().await?;

        Ok(PostFlowResult {
            data,
            cookies: if response_cookies.is_empty() {
                None
            } else {
                Some(response_cookies)
            },
        })
    }

    pub async fn register(
        &self,
        email: &str,
        username: &str,
        password: &str,
        geo_location: Option<&str>,
        cookie: Option<&str>,
    ) -> Result<(KratosIdentity, KratosSession), Box<dyn std::error::Error>> {
        let flow_result = self.fetch_flow("registration", cookie).await?;

        let mut traits = serde_json::json!({
            "email": email,
            "username": username
        });

        if let Some(geo) = geo_location {
            traits["geo_location"] = serde_json::json!(geo);
        }

        let registration_data = serde_json::json!({
            "method": "password",
            "traits": traits,
            "password": password,
            "csrf_token": flow_result.csrf_token,
        });

        let post_result = self
            .post_flow(
                "registration",
                &flow_result.flow["id"].as_str().unwrap(),
                registration_data,
                &flow_result.cookies,
            )
            .await?;

        let response_data = post_result.data;

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
                geo_location: response_data["identity"]["traits"]["geo_location"]
                    .as_str()
                    .map(|s| s.to_string()),
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

    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
        cookie: Option<&str>,
    ) -> Result<KratosSession, Box<dyn std::error::Error>> {
        let flow_result = self.fetch_flow("login", cookie).await?;

        let login_data = serde_json::json!({
            "method": "password",
            "identifier": identifier,
            "password": password,
            "csrf_token": flow_result.csrf_token,
        });

        let post_result = self
            .post_flow(
                "login",
                &flow_result.flow["id"].as_str().unwrap(),
                login_data,
                &flow_result.cookies,
            )
            .await?;

        let session_data = post_result.data;

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
