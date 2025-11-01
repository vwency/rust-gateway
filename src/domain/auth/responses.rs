use crate::infrastructure::adapters::kratos::kratos_client::KratosIdentity;
use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone)]
pub struct AuthResponse {
    pub session_token: String,
    pub user: UserView,
}

impl AuthResponse {
    pub fn from_kratos_identity(identity: KratosIdentity, session_token: String) -> Self {
        Self {
            session_token,
            user: UserView::from(identity),
        }
    }

    pub fn with_token(identity: KratosIdentity, token: String) -> Self {
        Self::from_kratos_identity(identity, token)
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

impl From<KratosIdentity> for UserView {
    fn from(identity: KratosIdentity) -> Self {
        Self {
            id: identity.id,
            email: identity.traits.email,
            login: identity.traits.username,
            created_at: identity.created_at,
            updated_at: identity.updated_at,
        }
    }
}
