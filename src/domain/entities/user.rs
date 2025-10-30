use crate::infrastructure::adapters::kratos::kratos_client::KratosIdentity;
use async_graphql::SimpleObject;

#[allow(unused)]
#[derive(SimpleObject, Clone)]
pub struct AuthResponse {
    pub session_token: String,
    pub user: UserView,
}

#[allow(unused)]
impl AuthResponse {
    pub fn from_kratos_identity(session_token: String, identity: KratosIdentity) -> Self {
        Self {
            session_token,
            user: UserView::from(identity),
        }
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
            created_at: identity.created_at.clone(),
            updated_at: identity.updated_at.clone(),
        }
    }
}
