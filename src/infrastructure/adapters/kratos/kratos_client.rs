use reqwest::StatusCode;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    pub active: bool,
    pub identity: KratosIdentity,
}

#[derive(Debug, Clone)]
pub struct FlowResult {
    pub flow: serde_json::Value,
    pub csrf_token: String,
    pub cookies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PostFlowResult {
    pub data: serde_json::Value,
    pub cookies: Vec<String>,
}

impl KratosClient {
    pub fn new(admin_url: String, public_url: String) -> Self {
        let client = Client::builder()
            .cookie_store(false)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .danger_accept_invalid_certs(true)
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
        endpoint: &str,
        cookie: Option<&str>,
    ) -> Result<FlowResult, Box<dyn std::error::Error>> {
        let url = format!("{}/self-service/{}/browser", self.public_url, endpoint);
        let url = url.replace("localhost", "127.0.0.1");

        let mut request = self.client.get(&url);

        if let Some(cookie_value) = cookie {
            request = request.header(header::COOKIE, cookie_value);
        }

        let response = request.send().await.map_err(|e| {
            format!(
                "Failed to connect to Kratos at {}: {}. Make sure Kratos is running.",
                url, e
            )
        })?;

        let status = response.status();
        let flow_cookies: Vec<String> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        if status == StatusCode::SEE_OTHER || status == StatusCode::FOUND {
            let location = response
                .headers()
                .get(header::LOCATION)
                .and_then(|h| h.to_str().ok())
                .ok_or("No redirect location found")?;

            let flow_id = location
                .split("flow=")
                .nth(1)
                .ok_or("Flow ID not found in redirect URL")?;

            let flow_url = format!(
                "{}/self-service/{}/flows?id={}",
                self.public_url.replace("localhost", "127.0.0.1"),
                endpoint,
                flow_id
            );

            let mut flow_request = self.client.get(&flow_url);

            if !flow_cookies.is_empty() {
                flow_request = flow_request.header(header::COOKIE, flow_cookies.join("; "));
            } else if let Some(cookie_value) = cookie {
                flow_request = flow_request.header(header::COOKIE, cookie_value);
            }

            let flow_response = flow_request.send().await?;

            if !flow_response.status().is_success() {
                let status = flow_response.status();

                let error_text = flow_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());

                return Err(format!(
                    "Failed to fetch {} flow (status {}): {}",
                    endpoint, status, error_text
                )
                .into());
            }

            let flow: serde_json::Value = flow_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse {} flow response: {}", endpoint, e))?;

            let csrf_token = flow["ui"]["nodes"]
                .as_array()
                .and_then(|nodes| {
                    nodes
                        .iter()
                        .find(|node| node["attributes"]["name"].as_str() == Some("csrf_token"))
                })
                .and_then(|node| node["attributes"]["value"].as_str())
                .ok_or("CSRF token not found in flow response")?
                .to_string();

            let mut all_cookies = Vec::new();
            if let Some(existing_cookie) = cookie {
                all_cookies.push(existing_cookie.to_string());
            }
            all_cookies.extend(flow_cookies);

            return Ok(FlowResult {
                flow,
                csrf_token,
                cookies: all_cookies,
            });
        }

        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Failed to fetch {} flow (status {}): {}",
                endpoint, status, error_text
            )
            .into());
        }

        let flow: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse {} flow response: {}", endpoint, e))?;

        let csrf_token = flow["ui"]["nodes"]
            .as_array()
            .and_then(|nodes| {
                nodes
                    .iter()
                    .find(|node| node["attributes"]["name"].as_str() == Some("csrf_token"))
            })
            .and_then(|node| node["attributes"]["value"].as_str())
            .ok_or("CSRF token not found in flow response")?
            .to_string();

        let mut all_cookies = Vec::new();
        if let Some(existing_cookie) = cookie {
            all_cookies.push(existing_cookie.to_string());
        }
        all_cookies.extend(flow_cookies);

        Ok(FlowResult {
            flow,
            csrf_token,
            cookies: all_cookies,
        })
    }

    async fn post_flow(
        &self,
        endpoint: &str,
        flow_id: &str,
        data: serde_json::Value,
        cookies: &[String],
    ) -> Result<PostFlowResult, Box<dyn std::error::Error>> {
        let cookie_header = cookies.join("; ");
        let url = format!(
            "{}/self-service/{}?flow={}",
            self.public_url, endpoint, flow_id
        );
        let url = url.replace("localhost", "127.0.0.1");

        let response = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::COOKIE, cookie_header)
            .json(&data)
            .send()
            .await
            .map_err(|e| format!("Failed to submit {} flow: {}", endpoint, e))?;

        let response_cookies: Vec<String> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("{} failed (status {}): {}", endpoint, status, error_text).into());
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse {} response: {}", endpoint, e))?;

        Ok(PostFlowResult {
            data,
            cookies: response_cookies,
        })
    }

    pub async fn handle_signup(
        &self,
        email: &str,
        username: &str,
        password: &str,
        cookie: Option<&str>,
    ) -> Result<(KratosSession, Vec<String>), Box<dyn std::error::Error>> {
        let flow_result = self.fetch_flow("registration", cookie).await?;

        let registration_data = serde_json::json!({
            "method": "password",
            "password": password,
            "traits": {
                "email": email,
                "username": username
            },
            "csrf_token": flow_result.csrf_token,
        });

        let post_result = self
            .post_flow(
                "registration",
                flow_result.flow["id"].as_str().ok_or("Flow ID not found")?,
                registration_data,
                &flow_result.cookies,
            )
            .await?;

        let session_data = &post_result.data["session"];

        let identity = KratosIdentity {
            id: session_data["identity"]["id"]
                .as_str()
                .ok_or("Identity ID not found")?
                .to_string(),
            schema_id: session_data["identity"]["schema_id"]
                .as_str()
                .unwrap_or("default")
                .to_string(),
            traits: IdentityTraits {
                email: session_data["identity"]["traits"]["email"]
                    .as_str()
                    .ok_or("Email not found")?
                    .to_string(),
                username: session_data["identity"]["traits"]["username"]
                    .as_str()
                    .ok_or("Username not found")?
                    .to_string(),
                geo_location: session_data["identity"]["traits"]["geo_location"]
                    .as_str()
                    .map(|s| s.to_string()),
            },
            created_at: session_data["identity"]["created_at"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            updated_at: session_data["identity"]["updated_at"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        };

        let session = KratosSession {
            id: session_data["id"]
                .as_str()
                .ok_or("Session ID not found")?
                .to_string(),
            active: session_data["active"].as_bool().unwrap_or(false),
            identity,
        };

        Ok((session, post_result.cookies))
    }

    pub async fn handle_login(
        &self,
        identifier: &str,
        password: &str,
        cookie: Option<&str>,
    ) -> Result<(KratosSession, Vec<String>), Box<dyn std::error::Error>> {
        let flow_result = self.fetch_flow("login", cookie).await?;

        let login_data = serde_json::json!({
            "method": "password",
            "password": password,
            "identifier": identifier,
            "csrf_token": flow_result.csrf_token,
        });

        let post_result = self
            .post_flow(
                "login",
                flow_result.flow["id"].as_str().ok_or("Flow ID not found")?,
                login_data,
                &flow_result.cookies,
            )
            .await?;

        let session_data = &post_result.data["session"];

        let identity = KratosIdentity {
            id: session_data["identity"]["id"]
                .as_str()
                .ok_or("Identity ID not found")?
                .to_string(),
            schema_id: session_data["identity"]["schema_id"]
                .as_str()
                .unwrap_or("default")
                .to_string(),
            traits: IdentityTraits {
                email: session_data["identity"]["traits"]["email"]
                    .as_str()
                    .ok_or("Email not found")?
                    .to_string(),
                username: session_data["identity"]["traits"]["username"]
                    .as_str()
                    .ok_or("Username not found")?
                    .to_string(),
                geo_location: session_data["identity"]["traits"]["geo_location"]
                    .as_str()
                    .map(|s| s.to_string()),
            },
            created_at: session_data["identity"]["created_at"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            updated_at: session_data["identity"]["updated_at"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        };

        let session = KratosSession {
            id: session_data["id"]
                .as_str()
                .ok_or("Session ID not found")?
                .to_string(),
            active: session_data["active"].as_bool().unwrap_or(false),
            identity,
        };

        Ok((session, post_result.cookies))
    }

    pub async fn handle_logout(
        &self,
        cookie: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("{}/self-service/logout/browser", self.public_url);
        let url = url.replace("localhost", "127.0.0.1");

        let flow_response = self
            .client
            .get(&url)
            .header(header::COOKIE, cookie)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to logout endpoint: {}", e))?;

        if !flow_response.status().is_success() {
            let error_text = flow_response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to get logout flow: {}", error_text).into());
        }

        let flow_data: serde_json::Value = flow_response.json().await?;
        let logout_url = flow_data["logout_url"]
            .as_str()
            .ok_or("Logout URL not found")?
            .replace("localhost", "127.0.0.1");

        let response = self
            .client
            .get(&logout_url)
            .header(header::COOKIE, cookie)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Logout failed: {}", error_text).into());
        }

        let cookies: Vec<String> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        Ok(cookies)
    }

    pub async fn handle_get_current_user(
        &self,
        cookie: &str,
    ) -> Result<IdentityTraits, Box<dyn std::error::Error>> {
        let url = format!("{}/sessions/whoami", self.public_url);
        let url = url.replace("localhost", "127.0.0.1");

        let response = self
            .client
            .get(&url)
            .header(header::COOKIE, cookie)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to whoami endpoint: {}", e))?;

        if !response.status().is_success() {
            return Err("Not logged in".into());
        }

        let session_data: serde_json::Value = response.json().await?;

        let traits = IdentityTraits {
            email: session_data["identity"]["traits"]["email"]
                .as_str()
                .ok_or("Email not found")?
                .to_string(),
            username: session_data["identity"]["traits"]["username"]
                .as_str()
                .ok_or("Username not found")?
                .to_string(),
            geo_location: session_data["identity"]["traits"]["geo_location"]
                .as_str()
                .map(|s| s.to_string()),
        };

        Ok(traits)
    }
}
