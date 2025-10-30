use ory_client::{
    apis::{configuration::Configuration, frontend_api, identity_api},
    models::{Identity, PerformNativeLogoutBody, Session, UpdateLoginFlowBody},
};

#[derive(Clone)]
pub struct KratosClient {
    config: Configuration,
    public_url: String,
}

#[derive(Debug, Clone)]
pub struct KratosAuthResult {
    pub identity: Identity,
    pub session: Session,
    pub session_token: String,
}

impl KratosClient {
    pub fn new(admin_url: String, public_url: String) -> Self {
        let mut config = Configuration::new();
        config.base_path = admin_url;

        Self { config, public_url }
    }

    pub async fn register(
        &self,
        email: &str,
        username: &str,
        password: &str,
        geo_location: Option<&str>,
    ) -> Result<KratosAuthResult, Box<dyn std::error::Error>> {
        let mut public_config = Configuration::new();
        public_config.base_path = self.public_url.clone();

        let registration_flow =
            frontend_api::create_native_registration_flow(&public_config, None, None, None, None)
                .await?;

        // Log the registration flow response for debugging
        println!("Registration flow response: {:?}", registration_flow);

        let csrf_token = registration_flow
            .ui
            .nodes
            .iter()
            .find_map(|node| {
                let attrs_json = serde_json::to_value(&*node.attributes).ok()?;
                let attrs = attrs_json.as_object()?;
                if attrs.get("name")?.as_str()? == "csrf_token" {
                    return attrs.get("value")?.as_str().map(|s| s.to_string());
                }
                None
            })
            .unwrap_or_default();

        let mut traits = serde_json::json!({
            "email": email,
            "username": username
        });

        if let Some(geo) = geo_location {
            traits["geo_location"] = serde_json::json!(geo);
        }

        let registration_body_json = serde_json::json!({
            "method": "password",
            "password": password,
            "traits": traits,
            "csrf_token": csrf_token,
        });

        let update_body = serde_json::from_value(registration_body_json)?;

        let result = frontend_api::update_registration_flow(
            &public_config,
            &registration_flow.id,
            update_body,
            None,
        )
        .await?;

        let session = match result.session {
            Some(session_box) => *session_box,
            None => return Err("No session returned".into()),
        };

        let session_token = match result.session_token {
            Some(token) => token,
            None => return Err("No session token returned".into()),
        };

        let identity_id = match session.identity.as_ref() {
            Some(identity) => identity.id.clone(),
            None => return Err("No identity in session".into()),
        };

        let identity = self.get_identity(&identity_id).await?;

        Ok(KratosAuthResult {
            identity,
            session,
            session_token,
        })
    }

    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<KratosAuthResult, Box<dyn std::error::Error>> {
        let mut public_config = Configuration::new();
        public_config.base_path = self.public_url.clone();

        let login_flow = frontend_api::create_native_login_flow(
            &public_config,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

        let csrf_token = login_flow
            .ui
            .nodes
            .iter()
            .find_map(|node| {
                let attrs_json = serde_json::to_value(&*node.attributes).ok()?;
                let attrs = attrs_json.as_object()?;
                if attrs.get("name")?.as_str()? == "csrf_token" {
                    return attrs.get("value")?.as_str().map(|s| s.to_string());
                }
                None
            })
            .unwrap_or_default();

        let login_body_json = serde_json::json!({
            "method": "password",
            "identifier": identifier,
            "password": password,
            "csrf_token": csrf_token,
        });

        let login_body: UpdateLoginFlowBody = serde_json::from_value(login_body_json)?;

        let result =
            frontend_api::update_login_flow(&public_config, &login_flow.id, login_body, None, None)
                .await?;

        let session = *result.session;
        let session_token = match result.session_token {
            Some(token) => token,
            None => return Err("No session token returned".into()),
        };

        let identity_id = match session.identity.as_ref() {
            Some(identity) => identity.id.clone(),
            None => return Err("No identity in session".into()),
        };

        let identity = self.get_identity(&identity_id).await?;

        Ok(KratosAuthResult {
            identity,
            session,
            session_token,
        })
    }

    pub async fn get_identity(
        &self,
        identity_id: &str,
    ) -> Result<Identity, Box<dyn std::error::Error>> {
        let identity = identity_api::get_identity(&self.config, identity_id, None).await?;
        Ok(identity)
    }

    pub async fn validate_session(
        &self,
        session_token: &str,
    ) -> Result<Session, Box<dyn std::error::Error>> {
        let mut config = self.config.clone();
        config.bearer_access_token = Some(session_token.to_string());

        let session = frontend_api::to_session(&config, Some(session_token), None, None).await?;

        Ok(session)
    }

    pub async fn logout(&self, session_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.config.clone();
        config.bearer_access_token = Some(session_token.to_string());

        let logout_body = PerformNativeLogoutBody {
            session_token: session_token.to_string(),
        };

        frontend_api::perform_native_logout(&config, logout_body).await?;
        Ok(())
    }
}
