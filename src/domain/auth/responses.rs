use crate::domain::entities::user::UserView;
use async_graphql::SimpleObject;

#[derive(SimpleObject, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserView,
}
