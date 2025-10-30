use crate::domain::entities::user::UserView;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: UserView) -> Result<UserView, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserView>, String>;
    async fn find_by_login(&self, login: &str) -> Result<Option<UserView>, String>;
    async fn find_by_id(&self, id: &str) -> Result<Option<UserView>, String>;
}
