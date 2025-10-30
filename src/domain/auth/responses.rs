use crate::infrastructure::adapters::kratos::kratos_client::KratosAuthResult;
use async_graphql::SimpleObject;
use ory_client::models::Identity;

#[derive(SimpleObject, Clone)]
pub struct AuthResponse {
    pub session_token: String,
    pub user: UserView,
}

impl AuthResponse {
    pub fn from_kratos_identity(session_token: String, identity: Identity) -> Self {
        let traits_value = identity.traits.expect("Expected traits to be an object");

        let traits = traits_value
            .as_object()
            .expect("Expected traits to be an object")
            .clone(); // <--- clone here

        Self {
            session_token,
            user: UserView {
                id: identity.id,
                email: traits
                    .get("email")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string(),
                login: traits
                    .get("username")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: identity.created_at.unwrap_or_default(),
                updated_at: identity.updated_at.unwrap_or_default(),
            },
        }
    }

    pub fn from_kratos_auth_result(result: KratosAuthResult) -> Self {
        Self::from_kratos_identity(result.session_token, result.identity)
    }
}

#[derive(SimpleObject, Clone)]
pub struct UserView {
    pub id: String,
    pub email: String,
    pub login: String,
    pub created_at: String,
    pub updated_at: String,
}
