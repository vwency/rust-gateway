use crate::infrastructure::adapters::kratos::kratos_client::KratosIdentity;
use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone)]
pub struct AuthResponse {
    // Убираем session_token из GraphQL ответа, так как он будет в cookie
    pub user: UserView,
}

impl AuthResponse {
    pub fn from_kratos_identity(identity: KratosIdentity) -> Self {
        Self {
            user: UserView::from(identity),
        }
    }

    // Добавляем метод для получения токена (используется внутри)
    pub fn with_token(identity: KratosIdentity, _token: String) -> (Self, String) {
        (Self::from_kratos_identity(identity), _token)
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
