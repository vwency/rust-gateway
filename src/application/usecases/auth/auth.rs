use ory_kratos_client::{
    apis::{
        configuration::Configuration,
        frontend_api::{
            create_browser_login_flow, create_browser_logout_flow, create_browser_recovery_flow,
            create_browser_registration_flow, to_session, update_login_flow, update_recovery_flow,
            update_registration_flow,
        },
    },
    models::{UpdateLoginFlowBody, UpdateRecoveryFlowBody, UpdateRegistrationFlowBody},
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct KratosConfig {
    pub base_path: String,
}

impl Default for KratosConfig {
    fn default() -> Self {
        let base_path =
            std::env::var("KRATOS_URL").unwrap_or_else(|_| "http://localhost:4433".to_string());

        tracing::info!("🔧 Using Kratos URL: {}", base_path);

        Self { base_path }
    }
}

impl KratosConfig {
    fn get_configuration(&self) -> Configuration {
        Configuration {
            base_path: self.base_path.clone(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session: serde_json::Value,
    pub session_cookie: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub identity: serde_json::Value,
    pub session_cookie: Option<String>,
}

// Объединенный use case для регистрации (signup)
pub struct Signup {
    email: String,
    password: String,
    traits: serde_json::Value,
    cookie: Option<String>,
    config: KratosConfig,
}

impl Signup {
    pub fn new(
        email: String,
        password: String,
        traits: serde_json::Value,
        cookie: Option<String>,
    ) -> Self {
        Self {
            email,
            password,
            traits,
            cookie,
            config: KratosConfig::default(),
        }
    }

    pub async fn execute(&self) -> Result<RegistrationResponse, Box<dyn std::error::Error>> {
        let configuration = self.config.get_configuration();

        tracing::info!("📝 Starting registration for email: {}", self.email);
        tracing::debug!("Cookie present: {}", self.cookie.is_some());

        // 1. Создаем registration flow
        tracing::info!("🔄 Creating browser registration flow...");
        let flow = match create_browser_registration_flow(
            &configuration,
            None,
            None,
            self.cookie.as_deref(),
            None,
        )
        .await
        {
            Ok(f) => {
                tracing::info!("✅ Registration flow created: {}", f.id);
                tracing::debug!("Flow details: {:?}", f);
                f
            }
            Err(e) => {
                tracing::error!("❌ Failed to create registration flow: {:?}", e);
                return Err(Box::new(e));
            }
        };

        // 2. Подготавливаем traits
        let mut traits_map = serde_json::Map::new();
        traits_map.insert(
            "email".to_string(),
            serde_json::Value::String(self.email.clone()),
        );

        if let serde_json::Value::Object(extra_traits) = &self.traits {
            for (k, v) in extra_traits {
                if k != "email" && k != "password" {
                    traits_map.insert(k.clone(), v.clone());
                }
            }
        }

        tracing::debug!("📋 Prepared traits: {:?}", traits_map);

        // 3. Формируем body для регистрации
        let body_json = json!({
            "method": "password",
            "password": self.password,
            "traits": serde_json::Value::Object(traits_map.clone()),
        });
        tracing::debug!(
            "📦 Request body (without password): {:?}",
            json!({
                "method": "password",
                "password": "***",
                "traits": serde_json::Value::Object(traits_map.clone()),
            })
        );
        let update_body: UpdateRegistrationFlowBody = serde_json::from_value(body_json)?;

        // 4. Завершаем регистрацию
        tracing::info!("🚀 Submitting registration flow: {}", flow.id);
        let response = match update_registration_flow(
            &configuration,
            &flow.id,
            update_body,
            self.cookie.as_deref(),
        )
        .await
        {
            Ok(r) => {
                tracing::info!("✅ Registration completed successfully");
                tracing::debug!("Response: {:?}", r);
                r
            }
            Err(e) => {
                tracing::error!("❌ Failed to update registration flow: {:?}", e);
                tracing::error!("Flow ID was: {}", flow.id);
                return Err(Box::new(e));
            }
        };

        // 5. Извлекаем session cookie из response headers (если есть)
        let session_cookie = response.session_token.clone();

        if session_cookie.is_some() {
            tracing::info!("🍪 Session cookie received");
        } else {
            tracing::warn!("⚠️ No session cookie in response");
        }

        Ok(RegistrationResponse {
            identity: serde_json::to_value(&response.identity)?,
            session_cookie,
        })
    }
}

// Объединенный use case для логина
pub struct Login {
    identifier: String,
    password: String,
    cookie: Option<String>,
    config: KratosConfig,
}

impl Login {
    pub fn new(identifier: String, password: String, cookie: Option<String>) -> Self {
        Self {
            identifier,
            password,
            cookie,
            config: KratosConfig::default(),
        }
    }

    pub async fn execute(&self) -> Result<LoginResponse, Box<dyn std::error::Error>> {
        let configuration = self.config.get_configuration();

        tracing::info!("🔐 Starting login for identifier: {}", self.identifier);

        // 1. Создаем login flow
        tracing::info!("🔄 Creating browser login flow...");
        let flow = match create_browser_login_flow(
            &configuration,
            None,
            None,
            None,
            self.cookie.as_deref(),
            None,
            None,
            None,
        )
        .await
        {
            Ok(f) => {
                tracing::info!("✅ Login flow created: {}", f.id);
                f
            }
            Err(e) => {
                tracing::error!("❌ Failed to create login flow: {:?}", e);
                return Err(Box::new(e));
            }
        };

        // 2. Формируем body для логина
        let body_json = json!({
            "method": "password",
            "identifier": self.identifier,
            "password": self.password,
        });

        let update_body: UpdateLoginFlowBody = serde_json::from_value(body_json)?;

        // 3. Завершаем логин
        tracing::info!("🚀 Submitting login flow: {}", flow.id);
        let response = match update_login_flow(
            &configuration,
            &flow.id,
            update_body,
            None,
            self.cookie.as_deref(),
        )
        .await
        {
            Ok(r) => {
                tracing::info!("✅ Login completed successfully");
                r
            }
            Err(e) => {
                tracing::error!("❌ Failed to update login flow: {:?}", e);
                return Err(Box::new(e));
            }
        };

        // 4. Извлекаем session cookie
        let session_cookie = response.session_token.clone();

        Ok(LoginResponse {
            session: serde_json::to_value(&response.session)?,
            session_cookie,
        })
    }
}

pub struct GetSession {
    cookie: String,
    config: KratosConfig,
}

impl GetSession {
    pub fn new(cookie: String) -> Self {
        Self {
            cookie,
            config: KratosConfig::default(),
        }
    }

    pub async fn execute(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let configuration = self.config.get_configuration();

        tracing::info!("👤 Getting session info");

        let session = match to_session(&configuration, None, Some(&self.cookie), None).await {
            Ok(s) => {
                tracing::info!("✅ Session retrieved successfully");
                s
            }
            Err(e) => {
                tracing::error!("❌ Failed to get session: {:?}", e);
                return Err(Box::new(e));
            }
        };

        Ok(serde_json::to_value(session)?)
    }
}

pub struct Logout {
    cookie: String,
    config: KratosConfig,
}

impl Logout {
    pub fn new(cookie: String) -> Self {
        Self {
            cookie,
            config: KratosConfig::default(),
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let configuration = self.config.get_configuration();

        tracing::info!("🚪 Starting logout");

        // Создаем logout flow
        let _logout_flow =
            match create_browser_logout_flow(&configuration, Some(&self.cookie), None).await {
                Ok(f) => {
                    tracing::info!("✅ Logout flow created successfully");
                    f
                }
                Err(e) => {
                    tracing::error!("❌ Failed to create logout flow: {:?}", e);
                    return Err(Box::new(e));
                }
            };

        // В Kratos logout происходит автоматически при создании logout flow
        Ok(())
    }
}

pub struct InitiateRecovery {
    cookie: Option<String>,
    config: KratosConfig,
}

impl InitiateRecovery {
    pub fn new(cookie: Option<String>) -> Self {
        Self {
            cookie,
            config: KratosConfig::default(),
        }
    }

    pub async fn execute(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let configuration = self.config.get_configuration();

        tracing::info!("🔑 Initiating password recovery");

        let flow = match create_browser_recovery_flow(&configuration, None).await {
            Ok(f) => {
                tracing::info!("✅ Recovery flow created: {}", f.id);
                f
            }
            Err(e) => {
                tracing::error!("❌ Failed to create recovery flow: {:?}", e);
                return Err(Box::new(e));
            }
        };

        Ok(serde_json::to_value(flow)?)
    }
}

pub struct CompleteRecovery {
    flow_id: String,
    email: String,
    cookie: Option<String>,
    config: KratosConfig,
}

impl CompleteRecovery {
    pub fn new(flow_id: String, email: String, cookie: Option<String>) -> Self {
        Self {
            flow_id,
            email,
            cookie,
            config: KratosConfig::default(),
        }
    }

    pub async fn execute(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let configuration = self.config.get_configuration();

        tracing::info!("🔑 Completing password recovery for: {}", self.email);

        let body_json = json!({
            "method": "link",
            "email": self.email,
        });

        let update_body: UpdateRecoveryFlowBody = serde_json::from_value(body_json)?;

        let response = match update_recovery_flow(
            &configuration,
            &self.flow_id,
            update_body,
            None,
            self.cookie.as_deref(),
        )
        .await
        {
            Ok(r) => {
                tracing::info!("✅ Recovery flow completed");
                r
            }
            Err(e) => {
                tracing::error!("❌ Failed to complete recovery flow: {:?}", e);
                return Err(Box::new(e));
            }
        };

        Ok(serde_json::to_value(response)?)
    }
}
