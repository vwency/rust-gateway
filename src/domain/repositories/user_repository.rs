use crate::domain::entities::user::User;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User) -> Result<User, String>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, String>;
    async fn find_by_login(&self, login: &str) -> Result<Option<User>, String>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, String>;
}
