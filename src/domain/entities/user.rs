use async_graphql::SimpleObject;

#[allow(unused)]
#[derive(SimpleObject, Clone)]
pub struct AuthResponse {
    pub session_token: String,
    pub user: UserView,
}

#[derive(SimpleObject, Clone)]
pub struct UserView {
    pub id: String,
    pub email: String,
    pub login: String,
    pub created_at: String,
    pub updated_at: String,
}
